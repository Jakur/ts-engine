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
            (winner, self.state.vp)
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
        );
        let us = &self.actors.us_agent;
        let (us_card, _us_eval) = us.decide_card(
            &self.state,
            self.state.deck.us_hand(),
            self.state.deck.china_available(Side::US),
        );
        // Headline order
        let pending = if us_card.ops() >= ussr_card.ops() {
            vec![Decision::new_event(ussr_card), Decision::new_event(us_card)]
        } else {
            vec![Decision::new_event(us_card), Decision::new_event(ussr_card)]
        };
        self.state.resolve_actions(&self.actors, pending);
    }
    fn final_scoring(&self) -> (Side, i8) {
        todo!()
    }
}
