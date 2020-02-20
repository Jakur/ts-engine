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
    fn step(&self, state: &mut GameState);
    /// Decides a country to act upon, or None if there is no legal option. Also
    /// includes a numerical evaluation of the position from the agent's perspective.
    fn decide_action(
        &self,
        state: &GameState,
        choices: &[usize],
        action: Action,
    ) -> (Option<usize>, f32);
    /// Picks a card among valid options for an action, either picking a card
    /// to play or else to discard. Also includes a numerical evaluation of the
    /// position from the agent's perspective.
    fn decide_card(
        &self,
        state: &GameState,
        cards: &[Card],
        china: bool,
        play: bool,
        can_pass: bool,
    ) -> (Option<Card>, f32);
    /// Returns just the evaluation of the given position
    fn get_eval(&self, state: &GameState) -> f32;
}

#[derive(Clone)]
pub struct DebugAgent<'a> {
    pub fav_action: Action,
    pub fav_card: Card,
    pub choices: &'a [usize],
}

impl<'a> DebugAgent<'a> {
    pub fn new(fav_action: Action, fav_card: Card, choices: &'a [usize]) -> Self {
        DebugAgent {
            fav_action,
            fav_card,
            choices,
        }
    }
}

impl<'a> Agent for DebugAgent<'a> {
    fn decide_action(&self, _s: &GameState, choices: &[usize], a: Action) -> (Option<usize>, f32) {
        use std::mem::discriminant;
        // Todo resolve first?
        let eval = if discriminant(&a) == discriminant(&self.fav_action) {
            1.0
        } else {
            0.0
        };
        for fav in self.choices.iter() {
            if choices.contains(fav) {
                return (Some(*fav), eval);
            }
        }
        let choice = if choices.len() != 0 {
            Some(choices[0])
        } else {
            None
        };
        (choice, eval)
    }
    fn decide_card(
        &self,
        _state: &GameState,
        _cards: &[Card],
        _china: bool,
        _play: bool,
        _can_pass: bool,
    ) -> (Option<Card>, f32) {
        todo!()
    }
    fn get_eval(&self, _state: &GameState) -> f32 {
        todo!()
    }
    fn step(&self, _: &mut GameState) { 
        unimplemented!() 
    }
}

pub struct RandAgent {}
impl RandAgent {
    pub fn new() -> Self {
        RandAgent {}
    }
}

impl Agent for RandAgent {
    fn decide_action(&self, _s: &GameState, choices: &[usize], _a: Action) -> (Option<usize>, f32) {
        if choices.len() == 0 {
            return (None, 0.0); // Todo detect this earlier?
        }
        let mut x = thread_rng();
        let choice = x.gen_range(0, choices.len());
        (Some(choices[choice]), x.gen())
    }
    fn decide_card(
        &self,
        _state: &GameState,
        hand: &[Card],
        china: bool,
        _play: bool,
        _can_pass: bool,
    ) -> (Option<Card>, f32) {
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
            (Some(card), x.gen())
        } else {
            let choices = &[None, Some(Card::The_China_Card)];
            // A player cannot be forced to play the China card
            if china {
                (choices[x.gen_range(0, 2)], x.gen())
            } else {
                (None, x.gen())
            }
        }
    }
    fn get_eval(&self, _state: &GameState) -> f32 {
        thread_rng().gen()
    }
    fn step(&self, state: &mut GameState) { 
        todo!()
    }
}
