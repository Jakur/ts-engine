use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;
use crate::tensor::{self, OutputVec};

use num_traits::FromPrimitive;

lazy_static!{
    static ref OFFSETS: Vec<usize> = {
        let mut vec = vec![0];
        let actions = (0..NUM_ACTIONS).into_iter().map(|x| Action::from_usize(x).unwrap());
        for a in actions {
            let last = *vec.last().unwrap();
            vec.push(a.legal_choices() + last);
        }
        vec
    };
}

pub const NUM_ACTIONS: usize = Action::Pass as usize + 1;

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
        Decision::new(card.side(), Action::Event, &[])
    }
    pub fn headline(agent: Side, state: &GameState) -> Self {
        let hand = state.deck.hand(agent);
        let vec: Vec<_> = hand.iter().filter_map(|c| {
            if c.can_headline(state) {
                Some(*c as usize)
            } else {
                None
            }
        }).collect();
        Decision::new(agent, Action::Event, vec)
    }
    pub fn new_no_allowed(agent: Side, action: Action) -> Decision {
        Decision::new(agent, action, &[])
    }
    pub fn new_standard(state: &GameState, agent: Side, action: Action, q: i8) -> Decision {
        let mut d = Decision::with_quantity(agent, action, &[], q);
        let allowed = state.standard_allowed(&d, &[]).expect("Make this always work");
        d.allowed = Allowed::new_owned(allowed);
        d
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
    pub fn next_decision(self) -> Option<Decision> {
        todo!()
        // If return none, reset limit in state
    }
    pub fn is_trivial(&self) -> bool {
        self.allowed.slice().len() <= 1
    }
}

fn standard_convert(action: &Action, slice: &[usize]) -> Vec<usize> {
    let offset = OFFSETS[*action as usize];
    slice.iter().map(|x| {
        offset + x
    }).collect()
}

#[derive(Clone, Copy, FromPrimitive)]
pub enum Action {
    BeginAr = 0,
    PlayCard,
    ConductOps,
    StandardOps, 
    Coup, 
    Space,
    Realignment,
    Place,      //Side, amount, can place in opponent controlled
    Remove,           // Side, remove all
    Discard,          // Side
    Event, // Card, Decision in branching events
    SpecialEvent,
    War, // Side, is brush war?
    Pass,
}

pub fn play_card_indices(state: &GameState) -> OutputVec {
    let f = play_card_index;
    let event_offset = Action::Event.offset();
    let hand = state.deck.hand(state.side);
    let mut vec = Vec::new();
    for &c in hand.iter() {
        let can_event = c.can_event(state);
        // Opponent Card
        if c.side() == state.side.opposite() {
            if can_event {
                vec.push(f(c, EventTime::Before));
                vec.push(f(c, EventTime::After));
            } else { 
                // Basically free ops in this case
                vec.push(f(c, EventTime::Never));
            }
        } else {
            // Event
            if can_event {
                vec.push(event_offset + c as usize);
            }
            // Play for ops
            if !c.is_scoring() {
                vec.push(f(c, EventTime::Never));
            }
        }
    }
    if state.deck.china_available(state.side) {
        let china = Card::The_China_Card;
        vec.push(f(china, EventTime::Never));
    }
    if hand.is_empty() || state.ar == 8 {
        vec.push(Action::Pass.offset()); // Pass
    }
    OutputVec::new(vec)
}

fn play_card_index(card: Card, resolve: EventTime) -> usize {
    (card as usize) * 3 + resolve as usize
}

impl Action {
    pub fn legal_choices(&self) -> usize {
        use Action::*;
        let countries = crate::country::NUM_COUNTRIES - 2;
        let cards = Card::total();
        match self {
            PlayCard => {
                // Todo if you really want to be precise you can make neutral special
                // For now we won't
                cards * 3
            },
            ConductOps | BeginAr => 0, // meta action or dummy
            StandardOps | Coup | Realignment | Place | Remove => countries,
            Space | Discard => cards,
            War => countries, // You can cut this down quite a bit as well
            Event => cards,
            SpecialEvent => *tensor::SPECIAL_TOTAL,
            Pass => 1,
        }
    }
    pub fn offset(&self) -> usize {
        OFFSETS[*self as usize]
    }
    pub fn action_index(data: usize) -> usize {
        let res = OFFSETS.binary_search(&data);
        match res {
            Ok(x) => x,
            Err(x) => x,
        }
    }
    pub fn action_from_offset(offset: usize) -> (Action, usize) {
        let index = Self::action_index(offset);
        let action = Action::from_usize(index).unwrap();
        let diff = offset - OFFSETS[index]; // Should be >= 0
        (action, diff)
    }
}

#[derive(Clone, FromPrimitive)]
pub enum EventTime {
    Before = 0,
    After = 1,
    Never = 2,
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
