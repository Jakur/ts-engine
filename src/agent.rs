use crate::action::Action;
use crate::card::Card;
use crate::state::GameState;

use rand::prelude::*;

pub trait Agent {
    fn decide_action(&self, state: &GameState, choices: &[usize], action: Action) -> (usize, f32);
    fn decide_card(&self, state: &GameState, hand: &[Card], china: bool) -> (Card, f32);
}

#[derive(Clone)]
pub struct RandAgent {}
impl RandAgent {
    pub fn new() -> Self {
        RandAgent {}
    }
}

impl Agent for RandAgent {
    fn decide_action(&self, _s: &GameState, choices: &[usize], _a: Action) -> (usize, f32) {
        let mut x = thread_rng();
        let choice = x.gen_range(0, choices.len());
        (choices[choice], x.gen())
    }
    fn decide_card(&self, _state: &GameState, hand: &[Card], china: bool) -> (Card, f32) {
        let mut x = thread_rng();
        let choice = if china {
            x.gen_range(0, hand.len() + 1)
        } else {
            x.gen_range(0, hand.len())
        };
        let card = if choice >= hand.len() {
            Card::The_China_Card
        } else {
            hand[choice]
        };
        (card, x.gen())
    }
}
