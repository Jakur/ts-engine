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
    pub fn restriction_clear() -> Decision<'a> {
        Decision::new(Side::Neutral, Action::ClearRestriction, &[])
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
    Place(Side, bool), //Side, can place in opponent controlled
    Remove(Side),
    Discard(Side, i8), // Side, ops minimum
    Event(Card, Option<usize>),
    ClearRestriction,
    AfterStates(Vec<Vec<Decision<'a>>>),
}

#[derive(Clone)]
pub enum Restriction {
    Limit(usize),
}
