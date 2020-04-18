use super::GameState;
use crate::action::{Action, Decision};
use crate::card::Effect;
use crate::country::Side;

pub struct Pending {
    pending: Vec<Decision>,
}

impl Pending {
    pub fn add(&mut self, state: &GameState, decision: Decision) {
        // After
        match decision.action {
            Action::Event => {
                self.pending
                    .push(Decision::new(Side::Neutral, Action::ClearEvent, &[]))
            }
            Action::BeginAr => self
                .pending
                .push(Decision::new(Side::Neutral, Action::EndAr, &[])),
        }

        self.pending.push(decision);

        // Before
        match decision.action {
            Action::Coup | Action::ConductOps => {
                if state.has_effect(decision.agent, Effect::CubanMissileCrisis) {
                    let legal = state.legal_cuban(decision.agent);
                    self.pending
                        .push(Decision::new(decision.agent, Action::CubanMissile, legal));
                }
            }
        }
    }
    pub fn remove(&mut self) -> Option<Decision> {
        self.pending.pop()
    }
    pub fn peek(&self) -> Option<&Decision> {
        self.pending.last()
    }
    pub fn peek_mut(&mut self) -> Option<&mut Decision> {
        self.pending.last_mut()
    }
}
