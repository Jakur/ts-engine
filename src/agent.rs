use crate::action::Action;
use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;

use rand::prelude::*;

pub struct Actors<A: Agent, B: Agent> {
    pub ussr_agent: A,
    pub us_agent: B,
}

impl<A, B> Actors<A, B>
where
    A: Agent,
    B: Agent,
{
    pub fn new(ussr_agent: A, us_agent: B) -> Actors<A, B> {
        Actors {
            ussr_agent,
            us_agent,
        }
    }
    pub fn get(&self, side: Side) -> &dyn Agent {
        match side {
            Side::USSR => &self.ussr_agent,
            Side::US => &self.us_agent,
            _ => unimplemented!(),
        }
    }
}

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
        if choices.len() == 0 {
            return (0, 0.0); // Todo detect this earlier?
        }
        let mut x = thread_rng();
        let choice = x.gen_range(0, choices.len());
        (choices[choice], x.gen())
    }
    fn decide_card(&self, _state: &GameState, hand: &[Card], china: bool) -> (Card, f32) {
        let mut x = thread_rng();
        if hand.len() > 0 {
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
        } else {
            let choices = &[Card::Pass, Card::The_China_Card];
            // A player cannot be forced to play the China card
            if china {
                (choices[x.gen_range(0, 2)], x.gen())
            } else {
                (Card::Pass, x.gen())
            }
        }
    }
}
