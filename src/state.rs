use crate::action::{Action, Decision, Restriction};
use crate::agent::{Agent, RandAgent};
use crate::card::*;
use crate::country::*;

use counter::Counter;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use std::collections::HashSet;

#[derive(Clone)]
pub struct GameState<'a> {
    pub countries: Vec<Country>,
    pub vp: i8,
    pub defcon: i8,
    turn: i8,
    ar: i8,
    side: Side,
    space: [i8; 2],
    mil_ops: [i8; 2],
    space_attempts: [i8; 2],
    pub us_effects: Vec<Effect>,
    pub ussr_effects: Vec<Effect>,
    pub pending_actions: Vec<Decision<'a>>, // Todo figure out actions
    pub rng: SmallRng,
    pub deck: Deck,
    pub restrict: Option<Restriction>,
    pub ussr_agent: RandAgent,
    pub us_agent: RandAgent,
}

impl<'a> GameState<'a> {
    pub fn new(map: &Map) -> GameState {
        GameState {
            countries: map.countries.clone(),
            vp: 0,
            defcon: 5,
            turn: 1, // Todo make compatible with initial placements
            ar: 1,
            side: Side::USSR,
            space: [0, 0],
            mil_ops: [0, 0],
            space_attempts: [0, 0],
            us_effects: Vec::new(),
            ussr_effects: Vec::new(),
            pending_actions: Vec::new(),
            rng: SmallRng::from_entropy(),
            deck: Deck::new(),
            restrict: None,
            ussr_agent: RandAgent::new(),
            us_agent: RandAgent::new(),
        }
    }
    pub fn roll(&mut self) -> i8 {
        self.rng.gen_range(1, 7)
    }
    pub fn standard_allowed(&self, side: Side, action: &Action) -> Option<Vec<usize>> {
        let allowed = match action {
            Action::StandardOps => Some(access(self, side)),
            Action::Coup(_, _) | Action::Realignment => {
                let opp = side.opposite();
                let valid = |v: &'static Vec<usize>| {
                    v.iter().filter_map(|x| {
                        if self.countries[*x].has_influence(opp) {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                };
                let mut vec: Vec<usize> = valid(&AFRICA).collect();
                vec.extend(valid(&CENTRAL_AMERICA));
                vec.extend(valid(&SOUTH_AMERICA));
                if self.defcon >= 3 {
                    vec.extend(valid(&MIDDLE_EAST));
                }
                if self.defcon >= 4 {
                    vec.extend(valid(&ASIA));
                }
                if self.defcon >= 5 {
                    vec.extend(valid(&EUROPE));
                }
                Some(vec)
            }
            _ => None,
        };
        allowed
    }
    pub fn resolve_actions(&mut self) -> f32 {
        let mut eval = 0.0;
        // Todo figure out how to get history out of local
        let mut history = Vec::new();
        while !self.pending_actions.is_empty() {
            let dec = self.pending_actions.pop().unwrap();
            // Todo don't compute this every time and see if dec.agent is right
            let mut computed_allowed = self.standard_allowed(dec.agent, &dec.action);
            if let Action::StandardOps = dec.action {
                // Check if we cannot break control
                let mut is_last_op = !self
                    .pending_actions
                    .last()
                    .and_then(|e| {
                        if let Action::StandardOps = e.action {
                            Some(true)
                        } else {
                            Some(false)
                        }
                    })
                    .unwrap_or(false);
                if is_last_op {
                    // Unwrap is safe because we always compute on StandardOps
                    computed_allowed = Some(
                        computed_allowed
                            .unwrap()
                            .into_iter()
                            .filter_map(|x| {
                                let c = &self.countries[x];
                                if c.controller() == dec.agent.opposite() {
                                    None
                                } else {
                                    Some(x)
                                }
                            })
                            .collect(),
                    );
                }
            }
            // Todo figure out if restriction is always limit
            if let Some(restrict) = &self.restrict {
                match restrict {
                    Restriction::Limit(num) => {
                        let counter: Counter<_> = history.iter().copied().collect();
                        let bad: HashSet<_> = counter
                            .into_map()
                            .into_iter()
                            .filter_map(|(k, v)| if v >= *num { Some(k) } else { None })
                            .collect();
                        if !bad.is_empty() {
                            // Todo: Under restrictions we should always use a reference slice?
                            computed_allowed = Some(
                                dec.allowed
                                    .iter()
                                    .filter_map(|x| if bad.contains(x) { None } else { Some(*x) })
                                    .collect(),
                            );
                        }
                    }
                }
            }
            let agent = match dec.agent {
                Side::US => &self.us_agent,
                Side::USSR => &self.ussr_agent,
                Side::Neutral => todo!(), // Events with no agency?
            };
            let decision = {
                match computed_allowed {
                    Some(vec) => agent.decide(&self, &vec[..], dec.action.clone()),
                    None => agent.decide(&self, dec.allowed, dec.action.clone()),
                }
            };
            let choice = decision.0;
            history.push(choice);
            eval = decision.1;
            match dec.action {
                Action::StandardOps => {
                    let cost = self.add_influence(dec.agent, choice);
                    if cost == 2 {
                        self.pending_actions.pop(); // The earlier check should apply
                    }
                }
                Action::Coup(ops, free) => {
                    let roll = self.roll();
                    self.take_coup(dec.agent, choice, ops, roll, free);
                }
                Action::Space => {
                    let roll = self.roll();
                    self.space_card(dec.agent, choice, roll);
                }
                Action::Event(card, _num) => {
                    card.event(self);
                }
                Action::Discard(side) => {
                    self.discard_card(side, choice);
                }
                Action::Remove(side) => {
                    self.remove_influence(side, choice);
                }
                Action::Realignment => {
                    let (us_roll, ussr_roll) = (self.roll(), self.roll());
                    self.take_realign(choice, us_roll, ussr_roll);
                }
                Action::Place(side) => {
                    self.add_influence(side, choice);
                }
                Action::AfterStates(mut acts) => {
                    // We can't be functional because float comparison is weird
                    let mut max_index = 0;
                    let mut max_val = 0.0;
                    for (i, v) in acts.iter().enumerate() {
                        let mut x = self.clone();
                        x.pending_actions.extend_from_slice(&v[..]);
                        let value = x.resolve_actions();
                        if value > max_val {
                            max_val = value;
                            max_index = i;
                        }
                    }
                    // Add the best found line to the actions queue to be executed
                    self.pending_actions.append(&mut acts[max_index]);
                }
                Action::ClearRestriction => {
                    history = Vec::new();
                    self.restrict = None;
                }
            }
        }
        return eval;
    }
    pub fn has_effect(&self, side: Side, effect: Effect) -> Option<usize> {
        let vec = match side {
            Side::US => &self.us_effects,
            Side::USSR => &self.ussr_effects,
            _ => unimplemented!(),
        };
        vec.iter().position(|e| *e == effect)
    }
    pub fn clear_effect(&mut self, side: Side, index: usize) {
        let vec = match side {
            Side::US => &mut self.us_effects,
            Side::USSR => &mut self.ussr_effects,
            _ => unimplemented!(),
        };
        vec.swap_remove(index);
    }
    pub fn is_controlled<T: Into<usize>>(&self, side: Side, country: T) -> bool {
        side == self.countries[country.into()].controller()
    }
    pub fn control(&mut self, side: Side, country: CName) {
        let c = &mut self.countries[country as usize];
        match side {
            Side::US => {
                c.us = std::cmp::max(c.us, c.ussr + c.stability);
            }
            Side::USSR => {
                c.ussr = std::cmp::max(c.ussr, c.us + c.stability);
            }
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn remove_influence(&mut self, side: Side, country_index: usize) {
        let c = &mut self.countries[country_index];
        // Require checking for influence prior to this step
        match side {
            Side::US => c.us -= 1,
            Side::USSR => c.ussr -= 1,
            _ => unimplemented!(),
        }
    }
    pub fn add_influence(&mut self, side: Side, country_index: usize) -> i8 {
        let c = &mut self.countries[country_index];
        let controller = c.controller();
        match side {
            Side::US => {
                c.us += 1;
                if controller == Side::USSR {
                    2
                } else {
                    1
                }
            }
            Side::USSR => {
                c.ussr += 1;
                if controller == Side::US {
                    2
                } else {
                    1
                }
            }
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn remove_all(&mut self, side: Side, country: CName) {
        let c = &mut self.countries[country as usize];
        match side {
            Side::US => {
                c.us = 0;
            }
            Side::USSR => {
                c.ussr = 0;
            }
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn take_realign(&mut self, country_index: usize, mut us_roll: i8, mut ussr_roll: i8) {
        // This should include superpowers as well
        for &c in EDGES[country_index].iter() {
            match self.countries[c].controller() {
                Side::US => us_roll += 1,
                Side::USSR => ussr_roll += 1,
                Side::Neutral => {}
            }
        }
        let country = &mut self.countries[country_index];
        match country.greater_influence() {
            Side::US => us_roll += 1,
            Side::USSR => ussr_roll += 1,
            Side::Neutral => {}
        }
        if us_roll > ussr_roll {
            let diff = us_roll - ussr_roll;
            country.ussr = std::cmp::max(0, country.ussr - diff);
        } else if ussr_roll > us_roll {
            let diff = ussr_roll - us_roll;
            country.us = std::cmp::max(0, country.us - diff);
        }
    }
    pub fn take_coup(&mut self, side: Side, country_index: usize, ops: i8, roll: i8, free: bool) {
        let c = &mut self.countries[country_index];
        let delta = std::cmp::max(0, ops + roll - 2 * c.stability);
        match side {
            Side::US => {
                let left = delta - c.ussr;
                if left > 0 {
                    c.ussr = 0;
                    c.us += left;
                } else {
                    c.ussr -= delta;
                }
            }
            Side::USSR => {
                let left = delta - c.us;
                if left > 0 {
                    c.us = 0;
                    c.ussr += left;
                } else {
                    c.us -= delta;
                }
            }
            Side::Neutral => unimplemented!(),
        }
        if c.bg {
            self.defcon -= 1;
        }
        if !free {
            let x = side as usize;
            self.mil_ops[x] = std::cmp::max(5, self.mil_ops[x] + ops);
        }
    }
    pub fn random_card(&mut self, side: Side) -> usize {
        // Todo figure out if this should end up in the Deck struct
        let start = {
            if side == self.deck.china() {
                1
            } else {
                0
            }
        };
        let end = match side {
            Side::US => self.deck.us_hand().len(),
            Side::USSR => self.deck.ussr_hand().len(),
            _ => unimplemented!(),
        };
        self.rng.gen_range(start, end)
    }
    pub fn discard_card(&mut self, side: Side, index: usize) {
        self.deck.play_card(side, index, false);
    }
    pub fn can_space(&self, side: Side) -> bool {
        let me = side as usize;
        let opp = side.opposite() as usize;
        self.space_attempts[me] < 1
            || self.space_attempts[me] < 2 && self.space[me] >= 2 && self.space[opp] < 2
    }
    pub fn space_card(&mut self, side: Side, index: usize, roll: i8) -> bool {
        let me = side as usize;
        let opp = side.opposite() as usize;
        self.deck.play_card(side, index, false);
        let success = match self.space[me] {
            0 | 2 | 4 | 6 => roll <= 3,
            1 | 3 | 5 => roll <= 4,
            7 => roll <= 2,
            _ => unimplemented!(),
        };
        self.space_attempts[me] += 1;
        if success {
            self.space[me] += 1;
            let first = self.space[me] > self.space[opp];
            let points = match self.space[me] {
                1 => {
                    if first {
                        2
                    } else {
                        1
                    }
                }
                3 => {
                    if first {
                        2
                    } else {
                        0
                    }
                }
                5 => {
                    if first {
                        3
                    } else {
                        1
                    }
                }
                7 => {
                    if first {
                        4
                    } else {
                        2
                    }
                }
                8 => {
                    if first {
                        2
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            match side {
                Side::US => self.vp += points,
                Side::USSR => self.vp -= points,
                _ => unimplemented!(),
            }
        }
        success
    }
    pub fn is_final_scoring(&self) -> bool {
        self.turn > 10
    }
}
