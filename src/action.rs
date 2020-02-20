use crate::card::Card;
use crate::country::Side;

#[derive(Clone)]
pub struct Decision {
    pub agent: Side,
    pub action: Action,
    pub allowed: Allowed,
    pub quantity: i8,
}

impl Decision {
    pub fn new<T>(agent: Side, action: Action, allowed: T) -> Decision 
        where T: Into<Allowed> {
        Decision::with_quantity(agent, action, allowed, 1)
    }
    pub fn with_quantity<T>(agent: Side, action: Action, allowed: T, q: i8) -> Decision
        where T: Into<Allowed> {
        Decision {
            agent,
            action,
            allowed: allowed.into(),
            quantity: q,
        }
    }
    pub fn use_card(agent: Side, action: Action) -> Decision {
        Decision::new(agent, action, &[])
    }
    pub fn new_event(card: Card) -> Self {
        Decision::new(card.side(), Action::Event(card, None), &[])
    }
    pub fn new_no_allowed(agent: Side, action: Action) -> Decision {
        Decision::new(agent, action, &[])
    }
    pub fn conduct_ops(agent: Side, ops: i8) -> Decision {
        // let d = Decision::new;
        // let inf = vec![d(agent, Action::StandardOps, &[]); ops as usize];
        // let coup = vec![d(agent, Action::Coup(ops, false), &[])];
        // let realign = vec![d(agent, Action::Realignment, &[]); ops as usize];
        // let vec = vec![inf, coup, realign];
        //d(agent, Action::AfterStates(vec), &[])
        todo!()
    }
    pub fn restriction_clear() -> Decision {
        unimplemented!()
    }
    pub fn limit_set(num: usize) -> Decision {
        unimplemented!()
    }
}

#[derive(Clone)]
pub enum Action {
    PlayCard(Card, EventTime),
    ConductOps,
    StandardOps,
    Coup(i8, bool), // Ops, Free
    Space(Card),
    Realignment,
    Place(Side, i8, bool),      //Side, amount, can place in opponent controlled
    Remove(Side, i8),           // Side, amount
    RemoveAll(Side, bool),      // Side, can remove in opponent controlled
    Discard(Side, Card),          // Side, card
    Event(Card, Option<usize>), // Card, Decision in branching events
    War(Side, bool), // Side, is brush war?
    Pass,
}

#[derive(Clone)]
pub enum EventTime {
    Before,
    After,
    Never
}

#[derive(Clone)]
pub enum Restriction {
    Limit(usize),
}

#[derive(Clone)]
/// Abstraction across data which is known at compile time and data that must be
/// computed on the fly.
pub struct Allowed {
    allowed: AllowedType
}

impl Allowed {
    pub fn new_slice(allowed: &'static [usize]) -> Allowed {
        let allowed = AllowedType::Slice(allowed);
        Allowed {allowed}
    }
    pub fn new_owned(allowed: Vec<usize>) -> Allowed {
        let allowed = AllowedType::Owned(allowed);
        Allowed {allowed}
    }
    pub fn slice(&self) -> &[usize] {
        match &self.allowed {
            AllowedType::Slice(s) => s,
            AllowedType::Owned(s) => &s,
            AllowedType::Empty => &[]
        }
    }
}

#[derive(Clone)]
enum AllowedType {
    Slice(&'static [usize]),
    Owned(Vec<usize>),
    Empty,
}

impl From<Vec<usize>> for Allowed {
    fn from(vec: Vec<usize>) -> Self {
        Allowed::new_owned(vec)
    }
}

impl From<&'static [usize]> for Allowed {
    fn from(slice: &'static [usize]) -> Self {
        Allowed::new_slice(slice)
    }
}

impl From<&[usize; 0]> for Allowed {
    fn from(_empty: &[usize; 0]) -> Self {
        let allowed = AllowedType::Empty;
        Allowed {allowed}
    }
}
