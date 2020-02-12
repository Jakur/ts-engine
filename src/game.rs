use crate::action::Decision;
use crate::agent::{Actors, Agent};
use crate::country::Side;
use crate::state::GameState;

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
        self.state.ar = 0;
        self.state.side = Side::USSR;
        self.headline();
        self.state.ar = 1;
        while self.state.ar <= goal_ar {
            // Todo AR 8 space power
            let mut pending = Vec::new();
            let card = self.state.choose_card(&self.actors);
            self.state.use_card(card, &mut pending);
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
        self.state.turn += 1;
        None
    }
    fn headline(&mut self) {
        // Todo see headline ability, can event card
        let ussr = &self.actors.ussr_agent;
        let (ussr_card, _ussr_eval) = ussr.decide_card(
            &self.state,
            self.state.deck.ussr_hand(),
            self.state.deck.china_available(Side::USSR),
            true,
        );
        let us = &self.actors.us_agent;
        let (us_card, _us_eval) = us.decide_card(
            &self.state,
            self.state.deck.us_hand(),
            self.state.deck.china_available(Side::US),
            true,
        );
        // Hands cannot be empty at the HL phase
        let us_card = us_card.unwrap();
        let ussr_card = ussr_card.unwrap();
        let decisions = (Decision::new_event(ussr_card), Decision::new_event(us_card));

        // Headline order
        if us_card.ops() >= ussr_card.ops() {
            self.state.side = Side::US;
            self.state.resolve_actions(&self.actors, vec![decisions.1]);
            self.state.side = Side::USSR;
            self.state.resolve_actions(&self.actors, vec![decisions.0]);
        } else {
            self.state.side = Side::USSR;
            self.state.resolve_actions(&self.actors, vec![decisions.0]);
            self.state.side = Side::US;
            self.state.resolve_actions(&self.actors, vec![decisions.1]);
        }
    }
    fn final_scoring(&self) -> (Side, i8) {
        todo!()
    }
}
