use crate::card::Card;
use crate::country::Side;

pub struct Decision {
    pub agent: Side,
    pub action: Action,
}

impl Action {
    pub fn new(agent: Side, action: Action) -> Decision {
        Decision { agent, action }
    }
}

pub enum Action {
    StandardOps(Side),
    Coup(Side, i8),
    Space(Side),
    Realignment(Side),
    Place(Side),
    FreeCoup(Side, i8), // Todo figure out if this should exist
    Remove(Side),
    Discard(Side, usize),
    Event(Card),
}
