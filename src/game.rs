use crate::action::{Action, Decision};
use crate::agent::{Actors, Agent};
use crate::card::Card;
use crate::country::Side;
use crate::state::{GameState, TwilightRand};
use crate::tensor::TensorOutput;

use std::mem;

pub struct Game<A: Agent, B: Agent, R: TwilightRand> {
    pub actors: Actors<A, B>,
    pub state: GameState,
    pub rng: R,
}

impl<A: Agent, B: Agent, R: TwilightRand > Game<A, B, R> {
    pub fn play(&mut self) -> (Side, i8) {
        self.initial_placement();
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
        let (_, us_choice) = us.decide(&self.state, us_decision.encode(&self.state));
        let us_card = Card::from_index(us_choice);

        let ussr = &self.actors.ussr_agent;
        let ussr_decision = Decision::headline(Side::USSR, &self.state);
        let (_, ussr_choice) = ussr.decide(&self.state, ussr_decision.encode(&self.state));
        let ussr_card = Card::from_index(ussr_choice); 

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
        while let Some(d) = pending.pop() {
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
            let choice = if d.is_trivial() {
                d.allowed.slice().iter().cloned().next()
            } else {
                let agent = self.actors.get(d.agent);
                let legal = d.encode(&self.state);
                let (action, choice) = agent.decide(&self.state, legal);
                match action {
                    Action::ConductOps | Action::BeginAr => {},
                    _ => assert_eq!(mem::discriminant(&action), mem::discriminant(&d.action)),
                }
                Some(choice)
            };
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
