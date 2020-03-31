use crate::action::{Action, Decision};
use crate::card::Card;
use crate::country::Side;
use crate::game::Game;
use crate::state::{DebugRand, GameState, TwilightRand};
use crate::tensor::{DecodedChoice, OutputIndex, OutputVec, TensorOutput};

use crossbeam_queue::ArrayQueue;
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
    pub choices: ArrayQueue<OutputIndex>,
}

impl ScriptedAgent {
    pub fn new(choices: &Vec<OutputIndex>) -> Self {
        let queue = ArrayQueue::new(choices.len());
        for c in choices.iter().copied() {
            queue.push(c).unwrap();
        }
        ScriptedAgent { choices: queue }
    }
    pub fn legal_line(&self, game: &mut Game<Self, Self, DebugRand>, goal_t: i8, goal_ar: i8) {
        let (_win, _pts) = game.play(goal_t, Some(goal_ar));
    }
    fn next(&self) -> Option<OutputIndex> {
        self.choices.pop().ok()
    }
}

impl Agent for ScriptedAgent {
    fn get_eval(&self, _state: &GameState) -> f32 {
        todo!()
    }
    fn decide(&self, _state: &GameState, legal: OutputVec) -> DecodedChoice {
        let next = self.next().unwrap();
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
