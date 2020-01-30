use crate::card::Card;
use crate::country::Side;

#[derive(Clone)]
pub struct Decision<'a> {
    pub agent: Side,
    pub action: Action,
    pub allowed: &'a [usize],
}

impl<'a> Decision<'a> {
    pub fn new(agent: Side, action: Action, allowed: &'a [usize]) -> Decision<'a> {
        Decision {
            agent,
            action,
            allowed,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Action {
    StandardOps,
    Coup(i8),
    Space,
    Realignment,
    Place(Side),
    FreeCoup(i8), // Todo figure out if this should exist
    Remove(Side),
    Discard(Side),
    Event(Card),
}

pub enum Restriction {
    Limit(i8),
}
