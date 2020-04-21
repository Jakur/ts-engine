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
        let act = decision.action;
        let side = decision.agent;
        match act {
            Action::Event => {
                self.pending
                    .push(Decision::new(Side::Neutral, Action::ClearEvent, &[]))
            }
            Action::BeginAr => self
                .pending
                .push(Decision::new(Side::Neutral, Action::EndAr, &[])),
            _ => {}
        }

        self.pending.push(decision);

        // Before
        match act {
            Action::Coup | Action::ConductOps => {
                if state.has_effect(side, Effect::CubanMissileCrisis) {
                    let legal = state.legal_cuban(side);
                    self.pending
                        .push(Decision::new(side, Action::CubanMissile, legal));
                }
            }
            _ => {}
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
