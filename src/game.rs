use crate::action::{Action, Decision};
use crate::agent::{Actors, Agent};
use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;
use crate::tensor::TensorOutput;

pub struct Game<A: Agent, B: Agent> {
    pub actors: Actors<A, B>,
    pub state: GameState,
}

impl<A: Agent, B: Agent> Game<A, B> {
    pub fn play(&mut self) -> (Side, i8) {
        self.state.initial_placement(&self.actors);
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
    fn do_turn(&mut self, goal_ar: i8) -> Option<Side> {
        use std::cmp::max;
        self.state.ar = 0;
        self.state.side = Side::USSR;
        self.headline();
        self.state.ar = 1;
        while self.state.ar <= goal_ar {
            // AR 8 space power
            // Todo North Sea Oil
            let mut can_pass = false;
            if self.state.ar == 8 {
                let space = self.state.space[self.state.side as usize];
                if space < 8 {
                    let win = self.state.advance_ply();
                    if win.is_some() {
                        return win;
                    }
                    continue;
                }
                can_pass = true;
            }
            let mut pending = Vec::new();
            let card = self.state.choose_card(&self.actors, can_pass);
            if let Some(c) = card {
                self.state.use_card(c, &mut pending);
            }
            self.state.resolve_actions(&self.actors, pending);
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
        None
    }
    fn headline(&mut self) {
        use crate::agent::legal_headline;
        // Todo see headline ability, can event card
        let get_card = |a: Action| {
            if let Action::Event(c) = a {
                c.expect("Works")
            } else {
                unreachable!()
            }
        };
        let us = &self.actors.us_agent;
        let us_legal = legal_headline(Side::US, &self.state);
        let (us_act, us_choice) = us.decide(&self.state, us_legal);
        let us_card = get_card(us_act);

        let ussr = &self.actors.ussr_agent;
        let ussr_legal = legal_headline(Side::USSR, &self.state);
        let (ussr_act, ussr_choice) = ussr.decide(&self.state, ussr_legal);
        let ussr_card = get_card(ussr_act); 

        // Hands cannot be empty at the HL phase
        let decisions = (Decision::new_event(ussr_card, &self.state), 
            Decision::new_event(us_card, &self.state));

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
        let mut history = Vec::new(); // Todo figure out how to handle this
        while let Some(d) = pending.pop() {
            let agent = self.actors.get(d.agent);
            let legal = d.encode();
            let (action, choice) = agent.decide(&self.state, legal);
            self.state.resolve_action(d, choice, &mut pending, &mut history);
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
