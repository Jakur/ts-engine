use crate::card::Card;
use crate::country::Side;

#[derive(Clone)]
pub struct Decision<'a> {
    pub agent: Side,
    pub action: Action<'a>,
    pub allowed: &'a [usize],
}

impl<'a> Decision<'a> {
    pub fn new(agent: Side, action: Action<'a>, allowed: &'a [usize]) -> Decision<'a> {
        Decision {
            agent,
            action,
            allowed,
        }
    }
    pub fn new_no_allowed(agent: Side, action: Action<'a>) -> Decision<'a> {
        Decision {
            agent,
            action,
            allowed: &[],
        }
    }
    pub fn conduct_ops(agent: Side, ops: i8) -> Decision<'a> {
        let d = Decision::new;
        let inf = vec![d(agent, Action::StandardOps, &[]); ops as usize];
        let coup = vec![d(agent, Action::Coup(ops, false), &[])];
        let realign = vec![d(agent, Action::Realignment, &[]); ops as usize];
        let vec = vec![inf, coup, realign];
        d(agent, Action::AfterStates(vec), &[])
    }
    pub fn restriction_clear() -> Decision<'a> {
        Decision::new(Side::Neutral, Action::ClearRestriction, &[])
    }
    pub fn limit_set(num: usize) -> Decision<'a> {
        Decision::new(Side::Neutral, Action::SetLimit(num), &[])
    }
}

#[derive(Clone)]
pub enum Action<'a> {
    StandardOps,
    ChinaInf,
    VietnamInf,
    Coup(i8, bool), // Ops, Free
    Space,
    Realignment,
    Place(Side, i8, bool), //Side, amount, can place in opponent controlled
    Remove(Side, i8),      // Side, amount
    RemoveAll(Side, bool), // Side, can remove in opponent controlled
    Discard(Side, i8),     // Side, ops minimum
    Event(Card, Option<usize>),
    ClearRestriction,
    AfterStates(Vec<Vec<Decision<'a>>>),
    War(Side, bool), // Side, is brush war?
    SetLimit(usize),
}

#[derive(Clone)]
pub enum Restriction {
    Limit(usize),
}
