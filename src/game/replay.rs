use super::{Game, Start, Win};
use crate::agent::{Agent, ScriptedAgent};
use crate::country::Side;
use crate::state::DebugRand;
use crate::state::GameState;
use crate::tensor::{DecodedChoice, OutputIndex, TensorOutput};

pub struct Replay {
    pub us_agent: ScriptedAgent,
    pub ussr_agent: ScriptedAgent,
    pub game: Game<DebugRand>,
    pub checks: Vec<Box<dyn Fn(&Self)>>,
    triggers: Vec<usize>,
    pub history: Vec<DecodedChoice>,
}

impl Replay {
    pub fn new(
        us_agent: ScriptedAgent,
        ussr_agent: ScriptedAgent,
        game: Game<DebugRand>,
        triggers: Vec<usize>,
    ) -> Self {
        Replay {
            us_agent,
            ussr_agent,
            game,
            checks: Vec::new(),
            triggers,
            history: Vec::new(),
        }
    }
    pub fn add_check(&mut self, check: Box<dyn Fn(&Self)>) {
        self.checks.push(check);
    }
    pub fn play(&mut self, start: Start) -> Option<Win> {
        self.checks.reverse();
        self.game.setup(start);
        while self.us_agent.choices.lock().unwrap().len() > 0
            || self.ussr_agent.choices.lock().unwrap().len() > 0
        {
            if let Some(t) = self.triggers.last() {
                if self.history.len() == *t {
                    let check = self.checks.pop().expect("Fewer checks than triggers!");
                    dbg!(*t);
                    check(&self);
                    self.triggers.pop();
                }
            }
            let next = self.game.state.peek_pending().unwrap();
            let agent = match next.agent {
                Side::US => &self.us_agent,
                Side::USSR => &self.ussr_agent,
                _ => unimplemented!(),
            };
            let decoded = if next.is_trivial() {
                let mut x = next.clone(); // This is cheap because next is trivial
                let legal = x.encode(&self.game.state);
                let action = legal.get(0).copied();
                let ret = action.unwrap_or(OutputIndex::pass()).decode();
                if agent.trivial_action(action) {
                    self.history.push(ret.clone())
                }
                ret
            } else {
                let legal = self.game.legal();
                let ret = agent.decide(&self.game.state, legal);
                self.history.push(ret.clone());
                ret
            };
            let res = self.game.consume_action(decoded);
            if let Err(win) = res {
                return Some(win);
            }
        }
        None
    }
}
