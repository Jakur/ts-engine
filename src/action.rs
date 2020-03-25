use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;
use crate::tensor::{self, OutputVec};

use num_traits::FromPrimitive;

lazy_static!{
    static ref OFFSETS: Vec<usize> = {
        let mut vec = vec![0];
        let actions = (0..NUM_ACTIONS - 1).into_iter().map(|x| Action::from_usize(x).unwrap());
        for a in actions {
            let last = *vec.last().unwrap();
            vec.push(a.legal_choices() + last);
        }
        vec
    };
}

pub const NUM_ACTIONS: usize = Action::Pass as usize + 1;

#[derive(Clone, Debug)]
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
    pub fn determine(agent: Side, action: Action, q: i8, state: &GameState) -> Decision {
        let allowed = match action {
            Action::StandardOps => state.legal_influence(agent, q),
            Action::Coup | Action::Realignment => state.legal_coup_realign(agent),
            _ => todo!(),
        };
        Decision::with_quantity(agent, action, allowed, q)
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
    /// Creates a (trivial) decision for eventing a single given card
    pub fn new_event(card: Card) -> Self {
        Decision::new(card.side(), Action::Event, vec![card as usize])
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
    pub fn begin_ar(agent: Side) -> Decision {
        Decision::new(agent, Action::BeginAr, &[])
    }
    pub fn new_no_allowed(agent: Side, action: Action) -> Decision {
        Decision::new(agent, action, &[])
    }
    pub fn new_standard(state: &GameState, agent: Side, action: Action, q: i8) -> Decision {
        let mut d = Decision::with_quantity(agent, action, &[], q);
        state.standard_allowed(&mut d, &[]);
        d
    }
    pub fn conduct_ops(agent: Side, ops: i8) -> Decision {
        Decision::with_quantity(agent, Action::ConductOps, &[], ops)
    }
    pub fn next_influence(mut self, state: &GameState) -> Option<Decision> {
        assert!(self.action == Action::StandardOps); // For now?
        if self.quantity == 0 {
            None
        } else if self.quantity == 1 {
            let opp = self.agent.opposite();
            let allowed: Vec<_> = self.allowed.slice().iter().copied().filter(|x| {
                !state.is_controlled(opp, *x)
            }).collect();
            let allowed = Allowed::new_owned(allowed);
            self.allowed = allowed;
            Some(self)
        } else {
            Some(self)
        }
    }
}

#[derive(Clone, Copy, FromPrimitive, Debug, PartialEq)]
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
    CubanMissile,
    RecoverCard, // SALT
    ChangeDefcon,
    Pass,
}

pub fn play_card_indices(agent: Side, state: &GameState) -> OutputVec {
    let f = play_card_index;
    let event_offset = Action::Event.offset();
    let hand = state.deck.hand(agent);
    let mut vec = Vec::new();
    for &c in hand.iter() {
        let can_event = c.can_event(state);
        // Opponent Card
        if c.side() == agent.opposite() {
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
    if state.deck.china_available(agent) {
        let china = Card::The_China_Card;
        vec.push(f(china, EventTime::Never));
    }
    if hand.is_empty() || state.ar == 8 {
        vec.push(Action::Pass.offset()); // Pass
    }
    OutputVec::new(vec)
}

pub fn play_card_index(card: Card, resolve: EventTime) -> usize {
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
            ConductOps | BeginAr => 1, // meta action or dummy
            StandardOps | Coup | Realignment | Place | Remove => countries,
            Space | Discard => cards,
            War => countries, // You can cut this down quite a bit as well
            Event => cards,
            SpecialEvent => *tensor::SPECIAL_TOTAL,
            CubanMissile => 3,
            RecoverCard => cards,
            ChangeDefcon => 6, // Todo avoid DEFCON 0? 
            Pass => 1,
        }
    }
    pub fn offset(&self) -> usize {
        OFFSETS[*self as usize]
    }
    pub fn play_card_data(choice: usize) -> (Card, EventTime) {
        let rem = choice % 3;
        let time = EventTime::from_usize(rem).unwrap();
        let card = Card::from_index(choice / 3);
        (card, time)
    }
    pub fn action_index(data: usize) -> usize {
        let res = OFFSETS.binary_search(&data);
        match res {
            Ok(x) => x,
            Err(x) => x - 1,
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

#[derive(Clone, Debug)]
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
    pub fn new_empty() -> Allowed {
        Allowed {allowed: AllowedType::Empty}
    }
    pub fn slice(&self) -> &[usize] {
        match &self.allowed {
            AllowedType::Slice(s) => s,
            AllowedType::Owned(s) => &s,
            AllowedType::Empty => &[]
        }
    }
}

#[derive(Clone, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::country::CName;
    #[test]
    fn test_action_offsets() {
        let mut last = 0;
        for i in 0..NUM_ACTIONS {
            let action = Action::from_usize(i).unwrap();
            let next = action.offset();
            if i == 0 {
                assert_eq!(next, 0);
            } else {
                assert!(next > last);
                last = next;
            }
        }
        let inf = Action::StandardOps; 
        let init_off = inf.offset(); 
        for &name in [CName::Turkey, CName::Austria, CName::Chile].iter() {
            let input = init_off + name as usize;
            let (act, c_index) = Action::action_from_offset(input);
            assert_eq!(name as usize, c_index);
            assert_eq!(inf, act);
        }
    }
}
