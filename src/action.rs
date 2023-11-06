use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;
use crate::tensor::{self, OutputVec};

use num_traits::FromPrimitive;

lazy_static! {
    static ref OFFSETS: Vec<usize> = {
        let mut vec = vec![0];
        let actions = (0..NUM_ACTIONS - 1)
            .into_iter()
            .map(|x| Action::from_usize(x).unwrap());
        for a in actions {
            let last = *vec.last().unwrap();
            vec.push(a.legal_choices() + last);
        }
        vec
    };
}

pub const NUM_ACTIONS: usize = Action::Pass as usize + 1;
pub const CUBAN_OFFSET: usize = Action::CubanMissile as usize;
pub const PASS: usize = Action::Pass as usize;

#[derive(Clone)]
pub struct Decision {
    pub agent: Side,
    pub action: Action,
    pub allowed: Allowed,
    pub quantity: i8,
}

impl std::fmt::Debug for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use crate::country::CName;
        use Action::*;
        let allowed_debug = match self.allowed.allowed {
            AllowedType::Owned(ref v) => {
                let out_vec: Vec<_> = match self.action {
                    Ops | OpsEvent | Event | EventOps | Space | Discard => v
                        .iter()
                        .map(|c| format!("{:?}", Card::from_index(*c)))
                        .collect(),
                    Influence | Coup | Realignment | Place | Remove | War => v
                        .iter()
                        .map(|c| format!("{:?}", CName::from_index(*c)))
                        .collect(),
                    _ => v.iter().map(|c| format!("{}", c)).collect(),
                };
                format!("{:?}", out_vec)
            }
            AllowedType::Slice(ref v) => {
                let out_vec: Vec<_> = match self.action {
                    Ops | OpsEvent | Event | EventOps | Space | Discard => v
                        .iter()
                        .map(|c| format!("{:?}", Card::from_index(*c)))
                        .collect(),
                    Influence | Coup | Realignment | Place | Remove | War => v
                        .iter()
                        .map(|c| format!("{:?}", CName::from_index(*c)))
                        .collect(),
                    _ => v.iter().map(|c| format!("{:?}", c)).collect(),
                };
                format!("{:?}", out_vec)
            }
            _ => format!("{:?}", self.allowed.allowed),
        };
        f.debug_struct("Decision")
            .field("agent", &self.agent)
            .field("action", &self.action)
            .field("allowed", &allowed_debug)
            .field("quantity", &self.quantity)
            .finish()
    }
}

impl Decision {
    pub fn new<T>(agent: Side, action: Action, allowed: T) -> Decision
    where
        T: Into<Allowed>,
    {
        Decision::with_quantity(agent, action, allowed, 1)
    }
    pub fn determine(agent: Side, action: Action, q: i8, state: &GameState) -> Decision {
        let allowed = match action {
            Action::Influence => state.legal_influence(agent, q),
            Action::Coup => state.legal_coup_realign(agent, true),
            Action::Realignment => state.legal_coup_realign(agent, false),
            _ => todo!(),
        };
        Decision::with_quantity(agent, action, allowed, q)
    }
    pub fn with_quantity<T>(agent: Side, action: Action, allowed: T, q: i8) -> Decision
    where
        T: Into<Allowed>,
    {
        let allowed = allowed.into();
        match action {
            Action::EndAr | Action::ClearEvent => assert_eq!(agent, Side::Neutral),
            Action::ConductOps | Action::BeginAr => match allowed.allowed {
                AllowedType::Unknown => {}
                _ => panic!("Cannot determine slice until OutputIndex resolution"),
            },
            _ => assert_ne!(agent, Side::Neutral),
        }
        Decision {
            agent,
            action,
            allowed,
            quantity: q,
        }
    }
    /// Creates a (trivial) decision for eventing a single given card
    pub fn new_event(caller: Side, card: Card) -> Self {
        Decision::new(caller, Action::Event, vec![card as usize])
    }
    pub fn is_single_event(&self) -> bool {
        if self.action != Action::Event {
            return false;
        }
        match self.allowed.allowed {
            AllowedType::Owned(ref vec) => vec.len() == 1,
            _ => false,
        }
    }
    pub fn is_defectors(&self) -> bool {
        if self.action != Action::Event {
            return false;
        }
        match self.allowed.allowed {
            AllowedType::Owned(ref vec) => vec.len() == 1 && vec[0] == Card::Defectors as usize,
            _ => false,
        }
    }
    pub fn is_trivial(&self) -> bool {
        match self.action {
            Action::BeginAr | Action::ConductOps => false,
            _ => {
                if let AllowedType::Lazy(_) = self.allowed.allowed {
                    false // Todo decide if we should expand this
                } else {
                    self.allowed.try_slice().unwrap().len() < 2
                }
            }
        }
    }
    pub fn headline(agent: Side, state: &GameState) -> Self {
        let hand = state.deck.hand(agent);
        let vec: Vec<_> = hand
            .iter_cards()
            .filter_map(|c| {
                if c.can_headline() {
                    Some(c as usize)
                } else {
                    None
                }
            })
            .collect();
        Decision::new(agent, Action::ChooseCard, vec)
    }
    pub fn begin_ar(agent: Side) -> Decision {
        Decision::new(agent, Action::BeginAr, Allowed::new_unknown())
    }
    pub fn new_no_allowed(agent: Side, action: Action) -> Decision {
        Decision::new(agent, action, &[])
    }
    pub fn conduct_ops(agent: Side, ops: i8) -> Decision {
        Decision::with_quantity(agent, Action::ConductOps, Allowed::new_unknown(), ops)
    }
    pub fn next_decision(
        mut self,
        history: &[tensor::DecodedChoice],
        state: &GameState,
    ) -> Option<Decision> {
        self.quantity -= 1;
        if self.quantity == 0 {
            return None;
        }
        match self.action {
            Action::Influence => self.next_influence(state),
            Action::Remove => {
                let changed_last = state.apply_restriction(history, &mut self);
                if !changed_last {
                    // Check if the country we just modified is now empty
                    if let Some(last) = history.last().map(|c| c.choice).flatten() {
                        let card = state.current_event().unwrap();
                        let country = &state.countries[last];
                        let period = state.period();
                        let (remove_side, _) = card.remove_quantity(self.agent, country, period);
                        if !state.countries[last].has_influence(remove_side) {
                            self.allowed = self
                                .allowed
                                .force_slice(state)
                                .iter()
                                .copied()
                                .filter(|x| *x != last)
                                .collect::<Vec<_>>()
                                .into();
                        }
                    }
                }
                Some(self)
            }
            Action::ChooseCard if state.current_event().unwrap() == Card::Ask_Not => {
                let mut allowed = vec![0];
                allowed.extend(state.deck.hand(Side::US).iter_cards().filter_map(|c| {
                    if let Card::Dummy = c {
                        None
                    } else {
                        Some(c as usize)
                    }
                }));
                self.allowed = Allowed::new_owned(allowed);
                Some(self)
            }
            _ => {
                state.apply_restriction(history, &mut self);
                Some(self)
            }
        }
    }
    pub fn next_influence(mut self, state: &GameState) -> Option<Decision> {
        assert!(self.action == Action::Influence); // For now?
        if self.quantity == 0 {
            None
        } else if self.quantity == 1 {
            let opp = self.agent.opposite();
            let allowed: Vec<_> = self
                .allowed
                .force_slice(state)
                .iter()
                .copied()
                .filter(|x| !state.is_controlled(opp, *x))
                .collect();
            let allowed = Allowed::new_owned(allowed);
            self.allowed = allowed;
            Some(self)
        } else {
            Some(self)
        }
    }
}

#[derive(Clone, Copy, FromPrimitive)]
pub enum CardUse {
    Influence,
    Coup,
    Space,
    Realignment,
}
// Todo do not ignore event first / event second

#[derive(Clone, Copy, FromPrimitive, Debug, PartialEq)]
pub enum Action {
    BeginAr = 0,
    EndAr,
    ClearEvent,
    ConductOps,
    Influence,
    Coup,
    Space,
    Realignment,
    Place,
    Remove,
    Discard,
    Ops,
    OpsEvent,
    EventOps,
    Event,
    SpecialEvent,
    War,
    CubanMissile,
    RecoverCard, // SALT
    ChangeDefcon,
    ChooseCard,  // Generic choice over cards for unusual events
    BlockRegion, // Chernobyl
    DoubleInf,   // LADS
    Pass,
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
            ConductOps | BeginAr | ClearEvent | EndAr => 1, // meta action or dummy
            Influence | Coup | Realignment | Place | Remove => countries,
            Space | Discard => cards,
            War => countries, // You can cut this down quite a bit as well
            Event | EventOps | Ops | OpsEvent => cards,
            SpecialEvent => *tensor::SPECIAL_TOTAL,
            CubanMissile => 3,
            RecoverCard => cards,
            ChangeDefcon => 6, // Todo avoid DEFCON 0?
            ChooseCard => cards,
            BlockRegion => 6,
            DoubleInf => crate::country::SOUTH_AMERICA.len(),
            Pass => 1,
        }
    }
    pub fn offset(&self) -> usize {
        OFFSETS[*self as usize]
    }
    pub fn from_index(index: usize) -> Action {
        Action::from_usize(index).unwrap()
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

#[derive(Clone, Debug)]
pub enum Restriction {
    Limit(usize),
}

#[derive(Clone, Debug)]
/// Abstraction across data which is known at compile time and data that must be
/// computed on the fly.
pub struct Allowed {
    allowed: AllowedType,
}

impl Allowed {
    pub fn new_slice(allowed: &'static [usize]) -> Allowed {
        let allowed = AllowedType::Slice(allowed);
        Allowed { allowed }
    }
    pub fn new_owned(allowed: Vec<usize>) -> Allowed {
        let allowed = AllowedType::Owned(allowed);
        Allowed { allowed }
    }
    pub fn new_empty() -> Allowed {
        Allowed {
            allowed: AllowedType::Empty,
        }
    }
    pub fn new_lazy(f: fn(&GameState) -> Vec<usize>) -> Allowed {
        Allowed {
            allowed: AllowedType::Lazy(f),
        }
    }
    pub fn new_unknown() -> Allowed {
        Allowed {
            allowed: AllowedType::Unknown,
        }
    }
    /// Attempts to slice allowed data that is currently readable.
    pub fn try_slice(&self) -> Option<&[usize]> {
        match &self.allowed {
            AllowedType::Slice(s) => Some(s),
            AllowedType::Owned(s) => Some(&s),
            AllowedType::Empty => Some(&[]),
            _ => None,
        }
    }
    /// Forces a resolution of problem cases. For lazy allowed, it resolves the
    /// laziness and converts the type appropriately. Panics on unknown allowed
    /// for meta types.
    pub fn force_slice(&mut self, state: &GameState) -> &[usize] {
        let mut resolved = self.resolve(state);
        if let Some(ref mut resolved) = resolved {
            std::mem::swap(self, resolved);
        }
        self.try_slice().unwrap()
    }
    fn force_iter<'a>(&'a self, state: &GameState) -> Box<dyn Iterator<Item = usize> + 'a> {
        //! This boxing incurs a small amount of overhead to avoid making the
        //! calling code explicitly handle both cases
        if let AllowedType::Lazy(f) = self.allowed {
            Box::new(f(state).into_iter())
        } else {
            Box::new(self.try_slice().unwrap().iter().copied())
        }
    }
    fn resolve(&self, state: &GameState) -> Option<Allowed> {
        if let AllowedType::Lazy(f) = self.allowed {
            Some(Allowed::new_owned(f(state)))
        } else {
            None
        }
    }
}

#[derive(Clone)]
enum AllowedType {
    Slice(&'static [usize]),
    Lazy(fn(&GameState) -> Vec<usize>),
    Owned(Vec<usize>),
    Empty,
    Unknown, // Unable to be read, used for meta types
}

impl std::fmt::Debug for AllowedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            AllowedType::Slice(s) => write!(f, "{:?}", s),
            AllowedType::Owned(v) => write!(f, "{:?}", v),
            AllowedType::Empty => write!(f, "[]"),
            AllowedType::Lazy(_) => write!(f, "LAZY"),
            AllowedType::Unknown => write!(f, "UNK"),
        }
    }
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
        Allowed { allowed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::country::CName;
    #[test]
    fn test_size() {
        dbg!(std::mem::size_of::<Allowed>());
        dbg!(std::mem::size_of::<Decision>());
    }
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
        let inf = Action::Influence;
        let init_off = inf.offset();
        for &name in [CName::Turkey, CName::Austria, CName::Chile].iter() {
            let input = init_off + name as usize;
            let (act, c_index) = Action::action_from_offset(input);
            assert_eq!(name as usize, c_index);
            assert_eq!(inf, act);
        }
    }
}
