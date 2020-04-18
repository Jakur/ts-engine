use crate::action::{Action, Decision};
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
    HL,
    AR,
}

pub struct Game<A: Agent, B: Agent, R: TwilightRand> {
    pub actors: Actors<A, B>,
    pub state: GameState,
    pub rng: R,
    pending_actions: Vec<Decision>,
    ply_history: Vec<DecodedChoice>,
    us_buf: Vec<DecodedChoice>,
    ussr_buf: Vec<DecodedChoice>,
}

impl<A: Agent, B: Agent, R: TwilightRand> Game<A, B, R> {
    pub fn new(ussr_agent: A, us_agent: B, state: GameState, rng: R) -> Game<A, B, R> {
        let actors = Actors::new(ussr_agent, us_agent);
        Game {
            actors,
            state,
            rng,
            pending_actions: Vec::new(),
            ply_history: Vec::new(),
            us_buf: Vec::new(),
            ussr_buf: Vec::new(),
        }
    }
    pub fn setup(&mut self) {
        // Todo figure this out
        self.state.deck.draw_cards(8, &mut self.rng);
        self.initial_placement();
    }
    pub fn consume_action(&mut self, decoded: DecodedChoice) -> Result<(), Win> {
        let goal = if self.state.turn <= 3 { 6 } else { 8 };

        let win = self.consume(decoded)?;
        if self.state.ar > goal {
            self.state.advance_turn()?;
            if self.state.turn > 10 {
                // Final scoring
                todo!()
            }
            self.pending_actions = self.hl_order();
        } else {
            if self.state.ar > self.goal_ar(self.state.side) {
                // Done for the turn
                self.state.advance_ply()?;
            }
        }
        Ok(())
    }
    fn consume(&mut self, decoded: DecodedChoice) -> Result<(), Win> {
        let mut decision = match self.pending_actions.pop() {
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
            // decision = Decision
        }
        // Headline
        if self.state.ar == 0 {
            if let Action::Event = decoded.action {
                // Undecided
                if decision.allowed.slice().len() > 1 {
                    let card = Card::from_index(choice.unwrap());
                    if self.pending_actions.last().unwrap().allowed.slice().len() > 1 {
                        // Both undecided
                        let second = Decision::new_event(decision.agent, card);
                        let first = self.pending_actions.pop().unwrap();
                        self.pending_actions = vec![second, first];
                        todo!()
                    } else {
                        // One has already decided, thus find resolution order
                        let d = Decision::new_event(decision.agent, card);
                        let d2 = self.pending_actions.pop().unwrap();
                        let card2 = Card::from_index(d2.allowed.slice()[0]);
                        let order = {
                            if card.base_ops() > card2.base_ops() {
                                vec![d2, d]
                            } else if card2.base_ops() > card.base_ops() {
                                vec![d, d2]
                            } else if d.agent == Side::US {
                                vec![d2, d]
                            } else {
                                vec![d, d2]
                            }
                        };
                        self.pending_actions = order;
                    }
                } else {
                    self.state.side = decision.agent; // Set phasing side
                }
            }
        }
        let next_d = self.state.resolve_action(
            decision,
            choice,
            &mut self.pending_actions,
            &mut self.ply_history,
            &mut self.rng,
        );
        if let Some(x) = next_d {
            // Still resolving parts of this action
            dbg!(&x);
            self.pending_actions.push(x);
        } else {
            while let Side::Neutral = decision.agent {
                self.state.resolve_neutral(decision.action)?;
            }
            // dbg!(&self.pending_actions);
            // if self.state.ar == 0 && !self.pending_actions.is_empty() {
            //     let win = self.state.check_win();
            //     return win;
            // }
            // if self.pending_actions.is_empty() {
            //     let win = self.state.advance_ply();
            //     dbg!(win);
            //     if self.state.ar > self.goal_ar() {
            //         let win = self.state.advance_turn();
            //         self.pending_actions = self.hl_order();
            //         return win;
            //     } else {
            //         self.pending_actions
            //             .push(Decision::begin_ar(self.state.side));
            //     }
            //     return win;
            // }
        }
        Ok(())
    }
    fn resolve_neutral(&mut self) -> Result<(), Win> {
        let decision = match self.pending_actions.pop() {
            Some(d) => d,
            _ => todo!(),
        };
        if let Side::Neutral = decision.agent {
            self.state.resolve_action(
                decision,
                None,
                &mut self.pending_actions,
                &mut self.ply_history,
                &mut self.rng,
            );
            return self.state.check_win();
        } else {
            panic!("Expected neutral side!");
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
    pub fn do_ply(&mut self) -> Option<Side> {
        let pending = vec![Decision::begin_ar(self.state.side)];
        self.resolve_actions(pending);
        let win = self.state.advance_ply();
        win
    }
    pub fn play(&mut self, goal_turn: i8, goal_ar: Option<i8>) -> (Side, i8) {
        // self.initial_placement();
        let mut instant_win = None;
        self.pending_actions = self.hl_order();
        while instant_win.is_none() && self.state.turn <= goal_turn {
            let next = self.pending_actions.last().unwrap();
            let agent = self.actors.get(next.agent);
            let legal = next.encode(&self.state);
            dbg!(&legal);
            let decoded = if legal.len() < 2 {
                let action = legal.get(0).copied();
                agent.trivial_action(action);
                action.unwrap_or(OutputIndex::pass()).decode()
            } else {
                agent.decide(&self.state, legal)
            };
            dbg!(&decoded);
            instant_win = self.consume_action(decoded);
            if let Some(winner) = instant_win {
                let amt = if let Side::US = winner { 20 } else { -20 };
                return (winner, amt);
            } else {
                while let Some(d) = self.pending_actions.last() {
                    if d.agent != Side::Neutral {
                        break;
                    }
                    instant_win = self.resolve_neutral();
                }
            }
            dbg!(instant_win);
        }
        if let Some(winner) = instant_win {
            // Always make instant wins 20 point victories
            if let Side::USSR = winner {
                (winner, -20)
            } else {
                (winner, 20)
            }
        } else {
            if self.state.turn >= 10 {
                self.final_scoring()
            } else {
                (Side::Neutral, self.state.vp)
            }
        }
    }
    fn initial_placement(&mut self) {
        use crate::country::{EASTERN_EUROPE, WESTERN_EUROPE};
        let mut pending_actions = Vec::new();
        // USSR
        let x = Decision::with_quantity(Side::USSR, Action::Place, &EASTERN_EUROPE[..], 6);
        pending_actions.push(x);
        self.resolve_actions(pending_actions);
        // US
        pending_actions = Vec::new();
        let x = Decision::with_quantity(Side::US, Action::Place, &WESTERN_EUROPE[..], 7);
        pending_actions.push(x);
        self.resolve_actions(pending_actions);
        // US Bonus + 2
        for _ in 0..2 {
            let mut pa = Vec::new();
            let mem: Vec<_> = self
                .state
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
                .collect();
            let dec = Decision::new(Side::US, Action::Place, mem);
            pa.push(dec);
            self.resolve_actions(pa);
        }
    }
    fn do_turn(&mut self, goal_ar: i8) -> Option<Side> {
        use std::cmp::max;

        if self.state.ar == 0 {
            self.state.side = Side::USSR;
            self.headline();
            self.state.ar = 1;
        }
        self.state.side = Side::USSR;
        while self.state.ar <= goal_ar {
            // AR 8 space power
            // Todo North Sea Oil
            if self.state.ar == 8 {
                let space = self.state.space[self.state.side as usize];
                if space < 8 {
                    let win = self.state.advance_ply();
                    if win.is_some() {
                        return win;
                    }
                    continue;
                }
            }
            let pending = vec![Decision::begin_ar(self.state.side)];
            self.resolve_actions(pending);
            let win = self.state.advance_ply();
            if win.is_some() {
                return win;
            }
        }
        let us_held = self.state.deck.held_scoring(Side::US);
        let ussr_held = self.state.deck.held_scoring(Side::USSR);
        // Holding cards is illegal, but it's possible in the physical game
        if us_held && ussr_held {
            return Some(Side::US); // US wins if both players cheat
        } else if us_held {
            return Some(Side::USSR);
        } else if ussr_held {
            return Some(Side::US);
        }
        // Mil ops
        let defcon = self.state.defcon;
        let us_pen = max(defcon - self.state.mil_ops[Side::US as usize], 0);
        let ussr_pen = max(defcon - self.state.mil_ops[Side::USSR as usize], 0);
        // These are penalties, so the signs are reversed from usual
        self.state.vp -= us_pen;
        self.state.vp += ussr_pen;
        self.state.turn += 1;
        // Reset Defcon and Mil ops for next turn
        self.state.defcon = std::cmp::min(defcon + 1, 5);
        self.state.mil_ops[0] = 0;
        self.state.mil_ops[1] = 0;
        // Check win before cleanup due to scoring cards held
        let win = self.state.check_win();
        self.state.deck.end_turn_cleanup();
        self.state.turn_effect_clear();
        win
    }
    fn headline(&mut self) {
        // Todo see headline ability, can event card
        let us = &self.actors.us_agent;
        let us_decision = Decision::headline(Side::US, &self.state);
        let us_decoded = us.decide(&self.state, us_decision.encode(&self.state));
        let us_card = Card::from_index(us_decoded.choice.unwrap());

        let ussr = &self.actors.ussr_agent;
        let ussr_decision = Decision::headline(Side::USSR, &self.state);
        let ussr_decoded = ussr.decide(&self.state, ussr_decision.encode(&self.state));
        let ussr_card = Card::from_index(ussr_decoded.choice.unwrap());

        // Hands cannot be empty at the HL phase
        let decisions = (
            Decision::new_event(Side::USSR, ussr_card),
            Decision::new_event(Side::US, us_card),
        );

        // Headline order
        if us_card.base_ops() >= ussr_card.base_ops() {
            self.state.side = Side::US;
            self.resolve_actions(vec![decisions.1]);
            self.state.side = Side::USSR;
            self.resolve_actions(vec![decisions.0]);
        } else {
            self.state.side = Side::USSR;
            self.resolve_actions(vec![decisions.0]);
            self.state.side = Side::US;
            self.resolve_actions(vec![decisions.1]);
        }
    }
    fn resolve_actions(&mut self, mut pending: Vec<Decision>) {
        use crate::card::Effect;
        let mut offered_cuban = false;
        while let Some(d) = pending.pop() {
            // let mut history = Vec::new(); // Todo figure out how to handle this
            // Cuban Missile Crisis
            if !offered_cuban {
                match &d.action {
                    Action::Coup | Action::ConductOps => {
                        if self.state.has_effect(d.agent, Effect::CubanMissileCrisis) {
                            let legal_cuban = self.state.legal_cuban(d.agent);
                            if !legal_cuban.slice().is_empty() {
                                let cuban_d =
                                    Decision::new(d.agent, Action::CubanMissile, legal_cuban);
                                pending.push(d);
                                pending.push(cuban_d);
                                offered_cuban = true;
                                continue;
                            }
                        }
                    }
                    _ => {}
                }
            }
            let mut decision = Some(d);
            while let Some(remaining) = decision {
                // todo!();
                decision = self.resolve_single(remaining);
                self.state.china = false; // Todo better place for this?
            }
        }
    }
    fn resolve_single(&mut self, mut decision: Decision) -> Option<Decision> {
        use crate::tensor::OutputIndex;

        let legal = decision.encode(&self.state);
        // Do not call eval if there are only 0 or 1 decisions
        let (action, choice) = if legal.len() <= 1 {
            let choice = decision.allowed.slice().iter().cloned().next();
            if decision.agent != Side::Neutral {
                let agent = self.actors.get(decision.agent);
                let index = choice.map(|i| OutputIndex::new(decision.action.offset() + i));
                agent.trivial_action(index);
            }
            (decision.action, choice)
        } else {
            let agent = self.actors.get(decision.agent);
            let x = agent.decide(&self.state, legal);
            (x.action, x.choice)
        };
        // Fix our decision if it was a meta decision that we're now collapsing
        if action != decision.action {
            let new_legal = match action {
                Action::Coup | Action::Realignment => self.state.legal_coup_realign(decision.agent),
                Action::Influence => self
                    .state
                    .legal_influence(decision.agent, decision.quantity),
                Action::Event | Action::EventOps | Action::Ops | Action::OpsEvent => Vec::new(), // Doesn't matter
                _ => unimplemented!(),
            };
            decision =
                Decision::with_quantity(decision.agent, action, new_legal, decision.quantity);
        }
        let res = self.state.resolve_action(
            decision,
            choice,
            &mut self.pending_actions,
            &mut self.ply_history,
            &mut self.rng,
        );
        res
    }
    fn final_scoring(&mut self) -> (Side, i8) {
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
            (Side::US, self.state.vp)
        } else {
            (Side::USSR, self.state.vp)
        }
    }
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
        game.pending_actions = vec![Decision::new(Side::USSR, Action::BeginAr, &[])];
        // game.pla
        assert_eq!(game.play(10, None).0, Side::US);
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
            pending_actions: Vec::new(),
            ply_history: Vec::new(),
        };
        game.setup();
        game
    }
    fn encode_inf(c_name: CName) -> OutputIndex {
        let offset = Action::Place.offset();
        OutputIndex::new(offset + (c_name as usize))
    }
}
