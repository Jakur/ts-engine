use crate::action::{Action, Decision};
use crate::agent::{Actors, Agent};
use crate::card::Card;
use crate::country::Side;
use crate::state::{GameState, TwilightRand};
use crate::tensor::{DecodedChoice, TensorOutput};

use std::mem;

pub struct Game<A: Agent, B: Agent, R: TwilightRand> {
    pub actors: Actors<A, B>,
    pub state: GameState,
    pub rng: R,
}

impl<A: Agent, B: Agent, R: TwilightRand > Game<A, B, R> {
    pub fn setup(&mut self) {
        // Todo figure this out
        self.initial_placement();
    }
    pub fn do_ply(&mut self) -> Option<Side> {
        let pending = vec![Decision::begin_ar(self.state.side)];
        self.resolve_actions(pending);
        let win = self.state.advance_ply();
        win
    }
    pub fn play(&mut self) -> (Side, i8) {
        // self.initial_placement();
        let mut instant_win = None;
        while instant_win.is_none() && self.state.turn <= 10 {
            // Todo add mid war / late war cards to deck
            instant_win = if self.state.turn <= 3 {
                self.do_turn(6)
            } else {
                self.do_turn(8) // Space race AR8 is impossible before Mid War
            };
        }
        if let Some(winner) = instant_win {
            // Always make instant wins 20 point victories
            if let Side::USSR = winner {
                (winner, -20)
            } else {
                (winner, 20)
            }
        } else {
            self.final_scoring()
        }
    }
    fn initial_placement(&mut self) {
        use crate::country::{WESTERN_EUROPE, EASTERN_EUROPE};
        let mut pending_actions = Vec::new();
        // USSR
        let x = Decision::with_quantity(
            Side::USSR,
            Action::Place,
            &EASTERN_EUROPE[..],
            6
        );
        pending_actions.push(x);
        self.resolve_actions(pending_actions);
        // US
        pending_actions = Vec::new();
        let x = Decision::with_quantity(
            Side::US, 
            Action::Place, 
            &WESTERN_EUROPE[..], 
            7
        );
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
        self.state.ar = 0;
        self.state.side = Side::USSR;
        self.headline();
        self.state.ar = 1;
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
        self.state.check_win()
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
        let decisions = (Decision::new_event(ussr_card), 
            Decision::new_event(us_card));

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
        let mut history = Vec::new(); // Todo figure out how to handle this
        let mut offered_cuban = false;
        let mut last_agent: Option<Side> = None;
        while let Some(mut d) = pending.pop() {
            // Reset current event 
            if let Some(last) = last_agent {
                if last != d.agent {
                    self.state.current_event = None;
                }
            }
            last_agent = Some(d.agent);
            // Cuban Missile Crisis 
            if !offered_cuban {
                match &d.action {
                    Action::Coup | Action::ConductOps => {
                        if self.state.has_effect(d.agent, Effect::CubanMissileCrisis) {
                            let legal_cuban = self.state.legal_cuban(d.agent);
                            if !legal_cuban.slice().is_empty() {
                                let cuban_d = Decision::new(d.agent, Action::CubanMissile, legal_cuban);
                                pending.push(d);
                                pending.push(cuban_d);
                                offered_cuban = true;
                                continue
                            }
                        }
                    },
                    _ => {},
                }
            }
            // Do not call eval if there are only 0 or 1 decisions
            let legal = d.encode(&self.state);
            let (action, choice) = if legal.is_trivial() {
                (d.action, d.allowed.slice().iter().cloned().next())
            } else {
                let agent = self.actors.get(d.agent);
                let x = agent.decide(&self.state, legal);
                // match action {
                //     Action::ConductOps | Action::BeginAr => {},
                //     _ => assert_eq!(mem::discriminant(&action), mem::discriminant(&d.action)),
                // }
                (x.action, x.choice)
            };
            // Fix our decision if it was a meta decision that we're now collapsing
            if action != d.action {
                let new_legal = match action {
                    Action::Coup | Action::Realignment => self.state.legal_coup_realign(d.agent),
                    Action::StandardOps => self.state.legal_influence(d.agent, d.quantity),
                    Action::Event => Vec::new(), // Doesn't matter
                    _ => unimplemented!(),
                };
                d = Decision::with_quantity(d.agent, action, new_legal, d.quantity);
            } 
            self.state.resolve_action(d, choice, &mut pending, &mut history, &mut self.rng);
        }
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
    use crate::tensor::OutputIndex;
    use crate::agent::*;
    use crate::state::DebugRand;
    use crate::country::CName;
    #[test]
    fn test_summit() {
        let mut game = standard_start();
        game.state.deck.us_hand_mut().extend(vec![Card::Summit; 7]);
        game.state.deck.ussr_hand_mut().extend(vec![Card::Summit; 7]);
        game.state.defcon = 2;
        let summit_play = OutputIndex::new(Action::Event.offset() + Card::Summit as usize);
        let x = summit_play.decode().action;
        assert_eq!(x, Action::Event);
        let defcon_one = OutputIndex::new(Action::ChangeDefcon.offset() + 1);
        let ussr = &mut game.actors.ussr_mut().choices;
        ussr.push(summit_play);
        let us = &mut game.actors.us_mut().choices;
        us.push(defcon_one);
        game.state.current_event = Some(Card::Summit); // Todo do this more nicely
        game.rng.rolls.append(&mut vec![5, 3]); // US, USSR 
        assert_eq!(game.do_ply().unwrap(), Side::US);
    }
    fn standard_start() -> Game<DebugAgent, DebugAgent, DebugRand> {
        use CName::*;
        let ussr = [Poland, Poland, Poland, Poland, EGermany, Austria];
        let us = [WGermany, WGermany, WGermany, WGermany, Italy, Italy, Italy,
            Italy, Iran];
        let ussr_agent = DebugAgent::new(ussr.iter().map(|c| encode_inf(*c)).collect());
        let us_agent = DebugAgent::new(us.iter().map(|c| encode_inf(*c)).collect());
        let actors = Actors::new(ussr_agent, us_agent);
        let state = GameState::new();
        let rng = DebugRand::new_empty();
        let mut game = Game {actors, state, rng};
        game.setup();
        game
    }
    fn encode_inf(c_name: CName) -> OutputIndex {
        let offset = Action::Place.offset();
        OutputIndex::new(offset + (c_name as usize))
    }
}
