use crate::action::{Action, Allowed, Decision};
use crate::agent::{Actors, Agent, ScriptedAgent};
use crate::card::Card;
use crate::country::Side;
use crate::state::{DebugRand, GameState, TwilightRand, Win};
use crate::tensor::{DecodedChoice, OutputIndex, TensorOutput};

#[derive(Clone, Copy)]
enum Blocked {
    US,
    USSR,
    Both, // Only possible in the headline
    Ready,
}

#[derive(Clone, Copy)]
enum Status {
    Start,
    ChooseHL,
    ResolveHL,
    AR,
}

pub struct Game<A: Agent, B: Agent, R: TwilightRand> {
    pub actors: Actors<A, B>,
    pub state: GameState,
    pub rng: R,
    ply_history: Vec<DecodedChoice>,
    us_buf: Vec<DecodedChoice>,
    ussr_buf: Vec<DecodedChoice>,
    status: Status,
}

impl<A: Agent, B: Agent, R: TwilightRand> Game<A, B, R> {
    pub fn new(ussr_agent: A, us_agent: B, state: GameState, rng: R) -> Game<A, B, R> {
        let actors = Actors::new(ussr_agent, us_agent);
        Game {
            actors,
            state,
            rng,
            ply_history: Vec::new(),
            us_buf: Vec::new(),
            ussr_buf: Vec::new(),
            status: Status::Start,
        }
    }
    pub fn setup(&mut self) {
        // Todo figure this out
        self.state.deck.draw_cards(8, &mut self.rng);
        self.initial_placement();
    }
    pub fn legal(&mut self) -> Vec<OutputIndex> {
        self.state.next_legal()
    }
    /// Consumes an incoming decoded choice from an agent, and resolves until
    /// either the game ends returning an Err(Win) or else until more input
    /// is needed from an agent returning Ok(vp_differential).
    pub fn consume_action(&mut self, decoded: DecodedChoice) -> Result<i8, Win> {
        let init_vp = self.state.vp;
        self.consume(decoded);
        self.resolve_neutral()?;
        self.update_status()?;
        Ok(self.state.vp - init_vp)
    }
    fn update_status(&mut self) -> Result<(), Win> {
        match self.status {
            Status::ChooseHL => {
                if self.state.pending().iter().all(|d| d.is_single_event()) {
                    let priority = |x: &Decision| {
                        let c = Card::from_index(x.allowed.simple_slice().unwrap()[0]);
                        2 * c.base_ops() + (Side::US == x.agent) as i8
                    };
                    self.status = Status::ResolveHL;
                    let d = self.state.remove_pending().unwrap();
                    let d2 = self.state.remove_pending().unwrap();
                    let order = if priority(&d) > priority(&d2) {
                        vec![d2, d]
                    } else {
                        vec![d, d2]
                    };
                    self.state.set_pending(order);
                    self.state.side = self.state.peek_pending().unwrap().agent;
                }
            }
            Status::ResolveHL => {
                if let Some(pending) = self.state.peek_pending() {
                    if pending.is_single_event() {
                        // Set phasing side
                        self.state.side = pending.agent;
                    }
                } else {
                    // Enter AR1
                    self.status = Status::AR;
                    self.state.check_win()?;
                    self.state.ar = 1;
                    self.state.add_pending(Decision::begin_ar(Side::USSR));
                }
            }
            Status::AR => {
                if self.state.empty_pending() {
                    self.state.advance_ply()?;
                    self.skip_null_ars()?;
                    let goal = if self.state.turn <= 3 { 6 } else { 8 };
                    if self.state.ar > goal {
                        self.state.advance_turn()?;
                        if self.state.turn > 10 {
                            return Err(self.final_scoring());
                        }
                        self.status = Status::ChooseHL;
                        self.state.set_pending(self.hl_order());
                    } else {
                        self.state.add_pending(Decision::begin_ar(self.state.side));
                    }
                }
            }
            Status::Start => {
                if self.state.empty_pending() {
                    // Todo this should need more to be accurate
                    self.status = Status::ChooseHL;
                    self.state.set_pending(self.hl_order());
                }
            }
        }
        Ok(())
    }
    fn skip_null_ars(&mut self) -> Result<(), Win> {
        let global_goal = if self.state.turn <= 3 { 6 } else { 8 };
        while self.state.ar <= global_goal {
            let side_goal = self.goal_ar(self.state.side);
            if self.state.ar > side_goal {
                self.state.advance_ply()?; // Skip this AR
            } else {
                return Ok(()); // Return control
            }
        }
        Ok(())
    }
    fn consume(&mut self, decoded: DecodedChoice) {
        let mut decision = match self.state.remove_pending() {
            Some(d) => d,
            _ => todo!(),
        };
        let choice = decoded.choice;
        if decoded.action != decision.action {
            // Todo clean this up, perhaps reapproaching it in a new way
            let lower = decoded.action.offset();
            let upper = Action::from_index(decoded.action as usize + 1).offset();
            let legal: Vec<_> = decision
                .encode(&self.state)
                .into_iter()
                .filter_map(|x| {
                    if lower <= x.inner() && x.inner() < upper {
                        Some(x.inner() - lower)
                    } else {
                        None
                    }
                })
                .collect();
            decision =
                Decision::with_quantity(decision.agent, decoded.action, legal, decision.quantity);
        }
        match self.status {
            Status::ChooseHL => {
                assert!(self.state.ar == 0);
                decision.allowed = Allowed::new_owned(vec![decoded.choice.unwrap()]);
                // Pretend we're FIFO
                let other = self.state.remove_pending().unwrap();
                self.state.add_pending(decision);
                self.state.add_pending(other);
            }
            Status::ResolveHL | Status::AR | Status::Start => {
                let next_d = self.state.resolve_action(
                    decision,
                    choice,
                    &mut self.ply_history,
                    &mut self.rng,
                );
                if let Some(x) = next_d {
                    // Still resolving parts of this action
                    dbg!(&x);
                    self.state.add_pending(x);
                }
            }
        }
    }
    fn resolve_neutral(&mut self) -> Result<(), Win> {
        while self.neutral_next() {
            let decision = self.state.remove_pending().unwrap();
            let next_d =
                self.state
                    .resolve_action(decision, None, &mut self.ply_history, &mut self.rng);
            assert!(next_d.is_none()); // Todo ensure this is the case
            self.state.check_win()?;
        }
        Ok(())
    }
    fn neutral_next(&self) -> bool {
        if let Some(side) = self.state.peek_pending().map(|d| d.agent) {
            side == Side::Neutral
        } else {
            false
        }
    }
    fn goal_ar(&self, side: Side) -> i8 {
        use crate::card::Effect;
        if self.state.turn <= 3 {
            6
        } else {
            let mut goal = 7;
            let us_space = self.state.space[Side::US as usize];
            let ussr_space = self.state.space[Side::USSR as usize];
            match side {
                Side::US => {
                    if (us_space == 8 && us_space > ussr_space)
                        || self.state.has_effect(Side::US, Effect::NorthSeaOil)
                    {
                        goal += 1;
                    }
                }
                Side::USSR => {
                    if ussr_space == 8 && ussr_space > us_space {
                        goal += 1;
                    }
                }
                _ => unimplemented!(),
            }
            goal
        }
    }
    fn hl_order(&self) -> Vec<Decision> {
        let us_hl = Decision::headline(Side::US, &self.state);
        let ussr_hl = Decision::headline(Side::USSR, &self.state);
        let us_space = self.state.space[Side::US as usize];
        let ussr_space = self.state.space[Side::USSR as usize];

        if ussr_space > us_space && ussr_space >= 4 {
            // If ussr has the power, explicitly make US input their decision first
            vec![ussr_hl, us_hl]
        } else {
            // Else assume as usual that the USSR is phasing first
            vec![us_hl, ussr_hl]
        }
    }
    pub fn play(&mut self, goal_turn: i8, goal_ar: Option<i8>) -> Result<(), Win> {
        // self.initial_placement();
        // Todo initial conditions
        if self.state.ar == 0 {
            self.state.set_pending(self.hl_order());
        }
        while self.state.turn <= goal_turn {
            let next = self.state.peek_pending().unwrap();
            let side = next.agent;
            let decoded = if next.is_trivial() {
                let mut x = next.clone(); // This is cheap because next is trivial
                let legal = x.encode(&self.state);
                let action = legal.get(0).copied();
                let agent = self.actors.get(next.agent);
                agent.trivial_action(action);
                action.unwrap_or(OutputIndex::pass()).decode()
            } else {
                let legal = self.legal();
                let agent = self.actors.get(side);
                agent.decide(&self.state, legal)
            };
            dbg!(&decoded);
            self.consume_action(decoded)?;

            while let Some(d) = self.state.peek_pending() {
                if d.agent != Side::Neutral {
                    break;
                }
                let pass = DecodedChoice::new(Action::Pass, None);
                self.consume_action(pass)?;
            }
        }
        if self.state.turn >= 10 {
            let res = self.final_scoring();
            return Err(res);
        }
        Ok(())
    }
    fn initial_placement(&mut self) {
        use crate::country::{EASTERN_EUROPE, WESTERN_EUROPE};
        let mut pending_actions = Vec::new();
        // USSR
        let x = Decision::with_quantity(Side::USSR, Action::Place, &EASTERN_EUROPE[..], 6);
        pending_actions.push(x);
        // US
        let x = Decision::with_quantity(Side::US, Action::Place, &WESTERN_EUROPE[..], 7);
        pending_actions.push(x);
        // US Bonus + 2
        for _ in 0..2 {
            let allowed = Allowed::new_lazy(legal_bonus_influence);
            let d = Decision::new(Side::US, Action::Place, allowed);
            pending_actions.push(d);
        }
        pending_actions = pending_actions.into_iter().rev().collect();
        self.state.set_pending(pending_actions);
    }
    fn final_scoring(&mut self) -> Win {
        use crate::country::Region::*;
        // Score Europe last, to let autoset to +20 vp
        let order = [
            Asia,
            MiddleEast,
            Africa,
            CentralAmerica,
            SouthAmerica,
            Europe,
        ];
        for r in order.iter() {
            r.score(&mut self.state);
        }
        if self.state.vp >= 0 {
            Win::Vp(Side::US)
        } else {
            Win::Vp(Side::USSR)
        }
    }
}

fn legal_bonus_influence(state: &GameState) -> Vec<usize> {
    state
        .valid_countries()
        .iter()
        .enumerate()
        .filter_map(|(i, x)| {
            // Apparently bonus influence cannot exceed stab + 2
            if x.us > 0 && x.us < x.stability + 2 {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::*;
    use crate::country::CName;
    use crate::state::DebugRand;
    use crate::tensor::OutputIndex;
    #[test]
    fn test_summit() {
        let mut game = standard_start();
        game.state.deck.us_hand_mut().extend(vec![Card::Summit; 7]);
        game.state
            .deck
            .ussr_hand_mut()
            .extend(vec![Card::Summit; 7]);
        game.status = Status::AR;
        game.state.defcon = 2;
        game.state.ar = 1;
        game.state.turn = 4;
        let summit_play = OutputIndex::new(Action::Event.offset() + Card::Summit as usize);
        let x = summit_play.decode().action;
        assert_eq!(x, Action::Event);
        let defcon_one = OutputIndex::new(Action::ChangeDefcon.offset() + 1);
        let ussr = &mut game.actors.ussr_mut().choices;
        ussr.lock().unwrap().push(summit_play);
        let us = &mut game.actors.us_mut().choices;
        us.lock().unwrap().push(defcon_one);
        game.rng.us_rolls = vec![5];
        game.rng.ussr_rolls = vec![3];
        // game.state.deck.ussr_hand_mut().push(Card::Summot)
        game.state
            .add_pending(Decision::new(Side::USSR, Action::BeginAr, &[]));
        // game.pla
        assert_eq!(game.play(10, None), Err(Win::Defcon(Side::US)));
    }
    fn test_traps() {
        // let mut game = standard_start();
        // game.state.deck.us_hand_mut().extend([Card::De_Gaulle_Leads_France, Card::Comecon].iter());
        // game.state.deck.us_hand_mut().extend(vec![Card::Summit; 5].iter());
        // game.state.deck.ussr_hand_mut().extend([Card::Olympic_Games, Card::NATO].iter());
        // game.state.deck.ussr_hand_mut().extend(vec![Card::Summit; 5].iter());
    }
    fn standard_start() -> Game<ScriptedAgent, ScriptedAgent, DebugRand> {
        use CName::*;
        let ussr = [Poland, Poland, Poland, Poland, EGermany, Austria];
        let us = [
            WGermany, WGermany, WGermany, WGermany, Italy, Italy, Italy, Italy, Iran,
        ];
        let ussr_agent = ScriptedAgent::new(&ussr.iter().map(|c| encode_inf(*c)).collect());
        let us_agent = ScriptedAgent::new(&us.iter().map(|c| encode_inf(*c)).collect());
        let actors = Actors::new(ussr_agent, us_agent);
        let state = GameState::new();
        let rng = DebugRand::new_empty();
        let mut game = Game {
            actors,
            state,
            rng,
            ply_history: Vec::new(),
            us_buf: Vec::new(),
            ussr_buf: Vec::new(),
            status: Status::Start,
        };
        game.setup();
        game
    }
    fn encode_inf(c_name: CName) -> OutputIndex {
        let offset = Action::Place.offset();
        OutputIndex::new(offset + (c_name as usize))
    }
}
