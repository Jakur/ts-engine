use crate::action::{Action, Decision, Restriction};
use crate::agent::{Agent, RandAgent};
use crate::card::*;
use crate::country::*;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

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
    pub china: Side,
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
            china: Side::USSR,
            restrict: None,
            ussr_agent: RandAgent::new(),
            us_agent: RandAgent::new(),
        }
    }
    pub fn standard_allowed(&self, side: Side, action: Action) -> Vec<usize> {
        let allowed = match action {
            Action::StandardOps => access(self, side),
            Action::Coup(_) | Action::FreeCoup(_) | Action::Realignment => {
                let opp = side.opposite();
                let action = |v: &'static Vec<usize>| {
                    v.iter().filter_map(|x| {
                        if self.countries[*x].has_influence(opp) {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                };
                let mut vec: Vec<usize> = action(&AFRICA).collect();
                vec.extend(action(&CENTRAL_AMERICA));
                vec.extend(action(&SOUTH_AMERICA));
                if self.defcon >= 3 {
                    vec.extend(action(&MIDDLE_EAST));
                }
                if self.defcon >= 4 {
                    vec.extend(action(&ASIA));
                }
                if self.defcon >= 5 {
                    vec.extend(action(&EUROPE));
                }
                vec
            }
            _ => vec![],
        };
        allowed
    }
    pub fn resolve_actions(&mut self) {
        let mut temp = Vec::new();
        while !self.pending_actions.is_empty() {
            let mut dec = self.pending_actions.pop().unwrap();
            if let Action::StandardOps = dec.action {
                // Check if we cannot break control
                if !self
                    .pending_actions
                    .last()
                    .and_then(|e| Some(e.action == Action::StandardOps))
                    .unwrap_or(false)
                {
                    temp = dec
                        .allowed
                        .iter()
                        .filter_map(|x| {
                            let c = &self.countries[*x];
                            if c.controller() == dec.agent.opposite() {
                                None
                            } else {
                                Some(*x)
                            }
                        })
                        .collect();
                    dec.allowed = &temp[..];
                }
            }
            let agent = match dec.agent {
                Side::US => &self.us_agent,
                Side::USSR => &self.ussr_agent,
                Side::Neutral => todo!(), // Events with no agency?
            };
            let choice = agent.decide(&self, dec.allowed, dec.action.clone());
            let choice = dec.allowed[choice];
            match dec.action {
                Action::StandardOps => {
                    let cost = self.add_influence(dec.agent, choice);
                    if cost == 2 {
                        self.pending_actions.pop(); // The earlier check should apply
                    }
                }
                _ => todo!(),
            }
        }
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
        let start = {
            if side == self.china {
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
        false // Todo fix this
    }
}
