use crate::action::{Decision, Action};
use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;
use crate::tensor::{TensorOutput, OutputVec, OutputIndex};

use rand::prelude::*;
use std::sync::{Arc, Mutex};

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
    /// Given a game state and encoding of all legal actions, decide which
    /// action to take and return the action the desired index
    fn decide(&self, state: &GameState, legal: OutputVec) -> (Action, usize);
    /// Returns which side the agent is playing
    fn side(&self) -> Side;
    /// Returns just the evaluation of the given position
    fn get_eval(&self, state: &GameState) -> f32;
}

#[derive(Clone)]
pub struct DebugAgent<'a> {
    pub ptr: Arc<Mutex<usize>>,
    pub choices: &'a [OutputIndex],
}

impl<'a> DebugAgent<'a> {
    pub fn new(choices: &'a [OutputIndex]) -> Self {
        DebugAgent {
            ptr: Arc::new(Mutex::new(0)),
            choices,
        }
    }
}

impl<'a> Agent for DebugAgent<'a> {
    fn get_eval(&self, _state: &GameState) -> f32 {
        todo!()
    }
    fn decide(&self, _state: &GameState, legal: OutputVec) -> (Action, usize) {
        let mut inner_ptr = self.ptr.lock().unwrap();
        let next = self.choices[*inner_ptr];
        *inner_ptr += 1;
        assert!(legal.contains(next));
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
    fn decide(&self, _state: &GameState, legal: OutputVec) -> (Action, usize) { 
        let mut rng = thread_rng();
        let x = legal.data().choose(&mut rng);
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
    let d = Decision::headline(agent, state);
    d.encode(state)
}
