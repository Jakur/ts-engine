use crate::card::Card;
use crate::country::Side;

#[derive(Clone)]
pub struct Decision<'a> {
    pub agent: Side,
    pub action: Action,
    pub allowed: &'a [usize],
    pub quantity: i8,
}

impl<'a> Decision<'a> {
    pub fn new(agent: Side, action: Action, allowed: &'a [usize]) -> Decision<'a> {
        Decision::with_quantity(agent, action, allowed, 1)
    }
    pub fn with_quantity(agent: Side, action: Action, allowed: &'a [usize], q: i8) -> Decision<'a> {
        Decision {
            agent,
            action,
            allowed,
            quantity: q,
        }
    }
    pub fn use_card(agent: Side, action: Action) -> Decision<'a> {
        Decision::new(agent, action, &[])
    }
    pub fn new_event(card: Card) -> Self {
        Decision::new(card.side(), Action::Event(card, None), &[])
    }
    pub fn new_no_allowed(agent: Side, action: Action) -> Decision<'a> {
        Decision {
            agent,
            action,
            allowed: &[],
            quantity: 1, // Todo fix
        }
    }
    pub fn conduct_ops(agent: Side, ops: i8) -> Decision<'a> {
        // let d = Decision::new;
        // let inf = vec![d(agent, Action::StandardOps, &[]); ops as usize];
        // let coup = vec![d(agent, Action::Coup(ops, false), &[])];
        // let realign = vec![d(agent, Action::Realignment, &[]); ops as usize];
        // let vec = vec![inf, coup, realign];
        //d(agent, Action::AfterStates(vec), &[])
        todo!()
    }
    pub fn restriction_clear() -> Decision<'a> {
        unimplemented!()
    }
    pub fn limit_set(num: usize) -> Decision<'a> {
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
        }
    }
}

enum AllowedType {
    Slice(&'static [usize]),
    Owned(Vec<usize>)
}
