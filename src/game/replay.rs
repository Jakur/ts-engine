use super::{Game, Start, Win};
use crate::agent::{Agent, ScriptedAgent};
use crate::country::Side;
use crate::state::DebugRand;
use crate::tensor::{OutputIndex, TensorOutput};

pub struct Replay {
    pub us_agent: ScriptedAgent,
    pub ussr_agent: ScriptedAgent,
    pub game: Game<DebugRand>,
}

impl Replay {
    pub fn new(us_agent: ScriptedAgent, ussr_agent: ScriptedAgent, game: Game<DebugRand>) -> Self {
        Replay {
            us_agent,
            ussr_agent,
            game,
        }
    }
    pub fn play(&mut self, start: Start) -> Option<Win> {
        self.game.setup(start);
        while self.us_agent.choices.lock().unwrap().len() > 0
            || self.ussr_agent.choices.lock().unwrap().len() > 0
        {
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
                agent.trivial_action(action);
                action.unwrap_or(OutputIndex::pass()).decode()
            } else {
                let legal = self.game.legal();
                agent.decide(&self.game.state, legal)
            };
            let res = self.game.consume_action(decoded);
            if let Err(win) = res {
                return Some(win);
            }
        }
        None
    }
}
