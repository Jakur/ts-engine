use crate::action::{Action, Allowed, Decision};
use crate::agent::{Actors, Agent, ScriptedAgent};
use crate::card::Card;
use crate::country::Side;
use crate::state::{DebugRand, GameState, TwilightRand, Win};
use crate::tensor::{DecodedChoice, OutputIndex, TensorOutput};

pub mod replay;

#[derive(Clone, Copy)]
enum Blocked {
    US,
    USSR,
    Both, // Only possible in the headline
    Ready,
}

#[derive(Clone, Copy, Debug)]
enum Status {
    Start,
    ChooseHL,
    ResolveHL,
    AR,
}
pub enum Start {
    Beginning,
    HL(i8),
    FirstAR(i8),
}

pub struct Game<R: TwilightRand> {
    pub state: GameState,
    pub rng: R,
    ply_history: Vec<DecodedChoice>,
    us_buf: Vec<DecodedChoice>,
    ussr_buf: Vec<DecodedChoice>,
    status: Status,
}

impl<R: TwilightRand> Game<R> {
    pub fn new(state: GameState, rng: R) -> Game<R> {
        Game {
            state,
            rng,
            ply_history: Vec::new(),
            us_buf: Vec::new(),
            ussr_buf: Vec::new(),
            status: Status::Start,
        }
    }
    pub fn four_four_two(&mut self) {
        use crate::country::CName;
        let c = &mut self.state.countries;
        c[CName::Italy as usize].us = 4;
        c[CName::WGermany as usize].us = 4;
        c[CName::Iran as usize].us = 2;
        c[CName::Poland as usize].ussr = 4;
        c[CName::EGermany as usize].ussr = 4;
        c[CName::Austria as usize].ussr = 1;
    }
    pub fn draw_hands(&mut self) {
        let goal = if self.state.turn <= 3 { 8 } else { 9 };
        self.state.deck.draw_cards(goal, &mut self.rng);
    }
    pub fn setup(&mut self, start: Start) {
        match start {
            Start::Beginning => {
                self.status = Status::Start;
                self.state.turn = 0;
                self.state.ar = 0;
                self.draw_hands();
                self.initial_placement();
            }
            Start::HL(turn) => {
                self.status = Status::ChooseHL;
                self.state.turn = turn;
                self.state.ar = 0;
                self.draw_hands();
                self.state.set_pending(self.hl_order());
            }
            Start::FirstAR(turn) => {
                self.status = Status::AR;
                self.state.turn = turn;
                self.state.ar = 1;
                self.state.side = Side::USSR;
                let d = Decision::begin_ar(Side::USSR);
                self.draw_hands();
                self.state.set_pending(vec![d])
            }
        }
    }
    pub fn legal(&mut self) -> Vec<OutputIndex> {
        self.state.next_legal()
    }
    /// Consumes an incoming decoded choice from an agent, and resolves until
    /// either the game ends returning an Err(Win) or else until more input
    /// is needed from an agent returning Ok(vp_differential).
    pub fn consume_action(&mut self, decoded: DecodedChoice) -> Result<i8, Win> {
        let init_vp = self.state.vp;
        // dbg!(&self.status);
        // dbg!(self.state.side);
        // dbg!(self.state.peek_pending());
        // dbg!(self.state.vp);
        // dbg!(self.legal());
        // dbg!(&decoded);
        self.consume(decoded);
        self.resolve_neutral()?;
        self.update_status()?;
        if self.state.turn > 10 {
            Err(self.final_scoring())
        } else {
            Ok(self.state.vp - init_vp)
        }
    }
    fn update_status(&mut self) -> Result<(), Win> {
        match self.status {
            Status::ChooseHL => {
                if self
                    .state
                    .pending()
                    .iter()
                    .all(|d| d.action != Action::ChooseCard)
                {
                    self.state.order_headlines();
                    self.status = Status::ResolveHL;
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
                    self.state.side = Side::USSR;
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
                        // Deck / Hand management
                        if self.state.turn == 4 {
                            self.state.deck.add_mid_war();
                        } else if self.state.turn == 7 {
                            self.state.deck.add_late_war();
                        }
                        self.draw_hands();
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
                    self.state.turn = 1;
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
        if decoded.action == Action::Pass {
            return;
        }
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
                let chosen_hl =
                    Decision::new(decision.agent, Action::Event, vec![decoded.choice.unwrap()]);

                if let Action::ChooseCard = self.state.peek_pending().unwrap().action {
                    // FIFO workaround
                    let pending_hl_choice = self.state.remove_pending().unwrap();
                    self.state.add_pending(chosen_hl);
                    self.state.add_pending(pending_hl_choice);
                } else {
                    self.state.add_pending(chosen_hl);
                }
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
                    // dbg!(&x);
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
    pub fn standard_start(rng: R) -> Game<R> {
        let state = GameState::four_four_two();
        let mut game = Game::new(state, rng);
        game.status = Status::ChooseHL;
        game.state.set_pending(game.hl_order());
        game
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
    use crate::game::replay::Replay;
    use crate::record::Record;
    use crate::state::DebugRand;
    #[test]
    fn test_summit() {
        let rng = DebugRand::new(vec![5], vec![3], Vec::new(), Vec::new(), Vec::new());
        let mut replay: Replay = Record::standard_start().into();
        let game = &mut replay.game;
        game.rng = rng;
        game.state.deck.us_hand_mut().extend(vec![Card::Summit; 7]);
        game.state
            .deck
            .ussr_hand_mut()
            .extend(vec![Card::Summit; 7]);
        game.status = Status::AR;
        game.state.set_defcon(2);
        game.state.ar = 1;
        game.state.turn = 4;
        game.state.clear_pending();
        game.state.add_pending(Decision::begin_ar(Side::USSR));
        let summit_play = DecodedChoice::new(Action::Event, Some(Card::Summit as usize));
        let defcon_one = DecodedChoice::new(Action::ChangeDefcon, Some(1));
        assert!(game.consume_action(summit_play).is_ok());
        assert_eq!(game.consume_action(defcon_one), Err(Win::Defcon(Side::US)));
    }
}
