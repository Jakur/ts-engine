use crate::action::Action;
use crate::state::GameState;

use rand::prelude::*;

pub trait Agent {
    fn decide(&self, state: &GameState, choices: &[usize], action: Action) -> (usize, f32);
}

#[derive(Clone)]
pub struct RandAgent {}
impl RandAgent {
    pub fn new() -> Self {
        RandAgent {}
    }
}

impl Agent for RandAgent {
    fn decide(&self, _state: &GameState, choices: &[usize], _action: Action) -> (usize, f32) {
        let mut x = thread_rng();
        let choice = x.gen_range(0, choices.len());
        (choices[choice], x.gen())
    }
}
