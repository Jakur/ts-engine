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
    Discard(Side),          // Side
    Event(Card, Option<usize>), // Card, Decision in branching events
    War(Side, bool), // Side, is brush war?
    IndependentReds, // No other event works like this 
    Destal, // Another special case
    Pass,
}

impl Action {
    pub fn dummy_actions() -> Vec<Action> {
        use Action::*;
        let c = Card::NATO;
        let s = Side::USSR;
        let mut vec = vec![PlayCard(c, EventTime::After), ConductOps, StandardOps, Coup(0, false), Space(c),
            Realignment, Place(s, 0, false), Remove(s, 0), RemoveAll(s, true), Discard(s),
            Event(c, None), War(s, false), IndependentReds, Destal, Pass];
        vec.sort_by_key(|a| a.index());
        vec
    }
    pub fn legal_choices(&self) -> usize {
        use Action::*;
        let countries = crate::country::NUM_COUNTRIES - 2;
        let cards = Card::total();
        match self {
            PlayCard(_, _) => {
                // Todo if you really want to be precise you can make neutral special
                // For now we won't
                cards * 3
            },
            ConductOps | RemoveAll(_, _) | Destal => 0, // meta action or dummy
            StandardOps | Coup(_, _) | Realignment | Place(_, _, _) | Remove(_, _) => countries,
            Space(_) | Discard(_) => cards,
            War(_, _) => countries, // You can cut this down quite a bit as well
            Event(_, _) => todo!(),
            IndependentReds => 5,
            Pass => 1,
        }
    }
    pub fn index(&self) -> usize {
        use Action::*;
        match self {
            PlayCard(_, _) => 0,
            ConductOps => 1,
            StandardOps => 2,
            Coup(_, _) => 3,
            Space(_) => 4,
            Realignment => 5,
            Place(_, _, _) => 6,
            Remove(_, _) => 7, 
            RemoveAll(_, _) => 8,    
            Discard(_) => 9, 
            Event(_, _) => 10,
            War(_, _) => 11,
            IndependentReds => 12,
            Destal => 13,
            Pass => 14,
        }
    }
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
