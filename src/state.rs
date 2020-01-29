use crate::action::Action;
use crate::card::*;
use crate::country::*;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

pub struct GameState {
    pub countries: Vec<Country>,
    pub vp: i8,
    pub defcon: i8,
    turn: i8,
    ar: i8,
    side: Side,
    ussr_space: i8,
    ussr_mil_ops: i8,
    us_space: i8,
    us_mil_ops: i8,
    pub us_effects: Vec<Effect>,
    pub ussr_effects: Vec<Effect>,
    pub pending_actions: Vec<Action>, // Todo figure out actions
    pub rng: SmallRng,
    pub us_hand: Vec<Card>,
    pub ussr_hand: Vec<Card>,
    pub china: Side,
}

impl GameState {
    pub fn new(map: &Map) -> GameState {
        GameState {
            countries: map.countries.clone(),
            vp: 0,
            defcon: 5,
            turn: 1, // Todo make compatible with initial placements
            ar: 1,
            side: Side::USSR,
            ussr_space: 0,
            ussr_mil_ops: 0,
            us_space: 0,
            us_mil_ops: 0,
            us_effects: Vec::new(),
            ussr_effects: Vec::new(),
            pending_actions: Vec::new(),
            rng: SmallRng::from_entropy(),
            us_hand: Vec::new(),
            ussr_hand: Vec::new(),
            china: Side::USSR,
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
    pub fn is_controlled(&self, side: Side, country: CName) -> bool {
        side == self.countries[country as usize].controller()
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
    pub fn random_card(&mut self, side: Side) -> usize {
        let start = {
            if side == self.china {
                1
            } else {
                0
            }
        };
        let end = match side {
            Side::US => self.us_hand.len(),
            Side::USSR => self.ussr_hand.len(),
            _ => unimplemented!(),
        };
        self.rng.gen_range(start, end)
    }
    pub fn discard_card(&mut self, side: Side, index: usize) {
        let hand = match side {
            Side::US => &mut self.us_hand,
            Side::USSR => &mut self.ussr_hand,
            _ => unimplemented!(),
        };
        hand.swap_remove(index);
    }
    pub fn is_final_scoring(&self) -> bool {
        false // Todo fix this
    }
}
