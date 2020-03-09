use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;
use crate::tensor::OutputVec;

lazy_static!{
    static ref OFFSETS: Vec<usize> = {
        let mut vec = vec![0];
        let actions = Action::dummy_actions();
        for a in actions {
            let last = *vec.last().unwrap();
            vec.push(a.legal_choices() + last);
        }
        vec
    };
}

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
    pub fn new_event(card: Card, state: &GameState) -> Self {
        let e_choices = card.e_choices(state);
        if let Some(choices) = e_choices {
            Decision::new(card.side(), Action::Event(Some(card)), choices)
        } else {
            Decision::new(card.side(), Action::Event(Some(card)), &[])
        }
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
        Decision::new(agent, Action::Event(None), vec)
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
    let offset = OFFSETS[action.index()];
    slice.iter().map(|x| {
        offset + x
    }).collect()
}

#[derive(Clone)]
pub enum Action {
    PlayCard,
    ConductOps(i8),
    StandardOps(i8),
    Coup(i8, bool), // Ops, Free
    Space,
    Realignment,
    Place(Side),      //Side, amount, can place in opponent controlled
    Remove(Side, bool),           // Side, remove all
    Discard(Side),          // Side
    Event(Option<Card>), // Card, Decision in branching events
    War(Side, bool), // Side, is brush war?
    Pass,
}

pub fn play_card_indices(state: &GameState) -> OutputVec {
    let f = play_card_index;
    let space_offset = OFFSETS[Action::Space.index()];
    let hand = state.deck.hand(state.side);
    let mut vec = Vec::new();
    let ops_offset = state.base_ops_offset(state.side);
    for &c in hand.iter() {
        let can_event = c.can_event(state);
        if c.side() == state.side.opposite() {
            if can_event {
                vec.push(f(c, EventTime::Before));
                vec.push(f(c, EventTime::After));
            } else { 
                // Basically free ops in this case
                vec.push(f(c, EventTime::Never));
            }
        } else {
            if c.is_scoring() {
                todo!();
                // vec.push(f(c, Some(0)));
                // continue;
            }
            // Play for ops
            vec.push(f(c, EventTime::Never));
            // Event
            if can_event {
                if let Some(chs) = c.e_choices(state) {
                    for x in chs {
                        todo!();
                        // vec.push(Action::Event(c, Some(x)));
                    }
                } else {
                    todo!();
                    // vec.push(Action::Event(c, Some(0)));
                }
            }
        }
        if state.can_space(state.side, c.base_ops() + ops_offset) {
            vec.push(space_offset + c as usize);
        }
    }
    if state.deck.china_available(state.side) {
        let china = Card::The_China_Card;
        vec.push(f(china, EventTime::Never));
        if state.can_space(state.side, china.base_ops() + ops_offset) {
            vec.push(space_offset + china as usize) // Legal, if not advisable
        }
    }
    if hand.is_empty() || state.ar == 8 {
        // vec.push(Action::Pass);
        vec.push(*OFFSETS.last().unwrap()); // Pass
    }
    OutputVec::new(vec)
}

fn play_card_index(card: Card, resolve: EventTime) -> usize {
    (card as usize) * 3 + resolve as usize
}

impl Action {
    pub fn dummy_actions() -> Vec<Action> {
        use Action::*;
        let c = Card::NATO;
        let s = Side::USSR;
        let mut vec = vec![PlayCard, ConductOps(2), StandardOps(2), Coup(0, false), Space,
            Realignment, Place(s), Remove(s, false), Discard(s),
            Event(None), War(s, false), Pass];
        vec.sort_by_key(|a| a.index());
        vec
    }
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
            ConductOps(_) => 0, // meta action or dummy
            StandardOps(_) | Coup(_, _) | Realignment | Place(_) | Remove(_, _) => countries,
            Space | Discard(_) => cards,
            War(_, _) => countries, // You can cut this down quite a bit as well
            Event(_) => todo!(),
            Pass => 1,
        }
    }
    pub fn offset(&self) -> usize {
        if let Action::Event(_) = self {
            todo!()
        }
        OFFSETS[self.index()]
    }
    fn index(&self) -> usize {
        use Action::*;
        match self {
            PlayCard => 0,
            ConductOps(_) => 1,
            StandardOps(_) => 2,
            Coup(_, _) => 3,
            Space => 4,
            Realignment => 5,
            Place(_) => 6,
            Remove(_, _) => 7,  
            Discard(_) => 8, 
            Event(_) => 9,
            War(_, _) => 10,
            Pass => 11,
        }
    }
}

#[derive(Clone)]
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
