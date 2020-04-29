use crate::action::{self, Decision};
use crate::country::Side;
use crate::game::Game;
use crate::state::{DebugRand, GameState};
use crate::tensor::{DecodedChoice, OutputIndex, OutputVec, TensorOutput};

use rand::prelude::*;
use std::sync::Mutex;

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
    pub fn ussr(&self) -> &A {
        &self.ussr_agent
    }
    pub fn ussr_mut(&mut self) -> &mut A {
        &mut self.ussr_agent
    }
    pub fn us(&self) -> &B {
        &self.us_agent
    }
    pub fn us_mut(&mut self) -> &mut B {
        &mut self.us_agent
    }
}

pub trait Agent {
    /// Given a game state and encoding of all legal actions, decide which
    /// action to take and return the action the desired index
    fn decide(&self, state: &GameState, legal: OutputVec) -> DecodedChoice;
    /// Returns which side the agent is playing
    fn side(&self) -> Side;
    /// Returns just the evaluation of the given position
    fn get_eval(&self, state: &GameState) -> f32;
}

pub struct ScriptedAgent {
    pub choices: Mutex<Vec<OutputIndex>>,
}

impl ScriptedAgent {
    pub fn new(choices: &Vec<OutputIndex>) -> Self {
        let v = choices.iter().copied().rev().collect();
        ScriptedAgent {
            choices: Mutex::new(v),
        }
    }
    /// Handles a trivial action, returning whether something was removed from the list
    pub fn trivial_action(&self, action: Option<OutputIndex>) -> bool {
        let mut choices = self.choices.lock().unwrap();
        let should_pop = match (choices.last(), action) {
            (Some(next), Some(taken)) => taken == *next,
            _ => false,
        };
        if should_pop {
            choices.pop();
        }
        should_pop
    }
    fn next(&self) -> Option<OutputIndex> {
        self.choices.lock().unwrap().pop()
    }
    fn peek(&self) -> Option<OutputIndex> {
        self.choices.lock().unwrap().last().map(|x| *x)
    }
}

impl Agent for ScriptedAgent {
    fn get_eval(&self, _state: &GameState) -> f32 {
        todo!()
    }
    fn decide(&self, _state: &GameState, legal: OutputVec) -> DecodedChoice {
        // Check for Cuban Missile Crisis action and Pass if we do not resolve it
        if 2 <= legal.len() && legal.len() <= 4 {
            let first_non = legal.iter().find(|x| x.inner() != action::PASS);
            if let Some(x) = first_non {
                if action::CUBAN_OFFSET <= x.inner() && x.inner() <= action::CUBAN_OFFSET + 3 {
                    let peek = self.peek().unwrap();
                    if legal.contains(&peek) {
                        return self.next().unwrap().decode();
                    } else {
                        return OutputIndex::new(action::PASS).decode();
                    }
                }
            }
        }
        let next = self.next().unwrap();
        if !legal.contains(&next) {
            dbg!(_state.ar);
            dbg!(legal);
            dbg!(next);
            panic!("Legal does not contain next!");
        }
        next.decode()
    }
    fn side(&self) -> Side {
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
    fn get_eval(&self, _state: &GameState) -> f32 {
        thread_rng().gen()
    }
    fn decide(&self, _state: &GameState, legal: OutputVec) -> DecodedChoice {
        let mut rng = thread_rng();
        let x = legal.choose(&mut rng);
        if let Some(choice) = x {
            choice.decode()
        } else {
            panic!("Nothing to decide!");
        }
    }
    fn side(&self) -> Side {
        unimplemented!()
    }
}

pub fn legal_headline(agent: Side, state: &GameState) -> OutputVec {
    let mut d = Decision::headline(agent, state);
    d.encode(state)
}
