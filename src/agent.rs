use crate::action::{Decision, Action};
use crate::card::Card;
use crate::country::Side;
use crate::state::{GameState, DebugRand, TwilightRand};
use crate::tensor::{TensorOutput, OutputVec, OutputIndex, DecodedChoice};

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

#[derive(Clone)]
pub struct DebugAgent {
    pub ptr: Arc<Mutex<usize>>,
    pub choices: Vec<OutputIndex>,
}

impl DebugAgent {
    pub fn new(choices: Vec<OutputIndex>) -> Self {
        DebugAgent {
            ptr: Arc::new(Mutex::new(0)),
            choices,
        }
    }
    pub fn legal_line(&self, state: &mut GameState, mut pending: Vec<Decision>, mut rng: DebugRand) -> bool {
        let mut history = Vec::new();
        while let (Some(decision), Some(next)) = (pending.pop(), self.choice()) {
            let legal = decision.encode(state);
            if !legal.contains(*next) {
                return false
            }
            let decoded = next.decode();
            state.resolve_action(decision, decoded.choice, &mut pending, &mut history, &mut rng);
            self.advance_ptr();
        }
        self.choice().is_none() && pending.is_empty()
    }
    fn choice(&self) -> Option<&OutputIndex> {
        let inner_ptr = self.ptr.lock().unwrap();
        let ret = self.choices.get(*inner_ptr);
        ret
    }
    fn advance_ptr(&self) {
        let mut inner_ptr = self.ptr.lock().unwrap();
        *inner_ptr += 1;
    }
    fn next(&self) -> Option<&OutputIndex> {
        let mut inner_ptr = self.ptr.lock().unwrap();
        let ret = self.choices.get(*inner_ptr);
        *inner_ptr += 1;
        ret
    }
}

impl Agent for DebugAgent {
    fn get_eval(&self, _state: &GameState) -> f32 {
        todo!()
    }
    fn decide(&self, _state: &GameState, legal: OutputVec) -> DecodedChoice {
        let next = self.next().unwrap();
        assert!(legal.contains(*next));
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
