use crate::action::{Action, Allowed, Decision, EventTime, Restriction};
use crate::card::*;
use crate::country::*;
use crate::tensor::DecodedChoice;

use counter::Counter;

use std::collections::HashSet;
mod random;
pub use random::{DebugRand, InternalRand, TwilightRand};

#[derive(Clone)]
pub struct GameState {
    pub countries: Vec<Country>,
    pub vp: i8,
    pub defcon: i8,
    pub turn: i8,
    pub ar: i8,
    pub side: Side,
    pub space: [i8; 2],
    pub mil_ops: [i8; 2],
    space_attempts: [i8; 2],
    pub us_effects: Vec<Effect>,
    pub ussr_effects: Vec<Effect>,
    pub deck: Deck,
    pub restrict: Option<Restriction>,
    pub current_event: Option<Card>,
    pub vietnam: bool,
    pub china: bool,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            countries: standard_start(),
            vp: 0,
            defcon: 5,
            turn: 1, // Todo make compatible with initial placements
            ar: 0,
            side: Side::USSR,
            space: [0, 0],
            mil_ops: [0, 0],
            space_attempts: [0, 0],
            us_effects: Vec::new(),
            ussr_effects: Vec::new(),
            deck: Deck::new(),
            restrict: None,
            current_event: None,
            vietnam: false,
            china: false,
        }
    }
    pub fn four_four_two() -> GameState {
        use crate::country::CName::*;
        let mut state = GameState::new();
        let c = &mut state.countries;
        c[Italy as usize].us = 4;
        c[WGermany as usize].us = 4;
        c[Iran as usize].us = 2;
        c[EGermany as usize].ussr = 4;
        c[Poland as usize].ussr = 4;
        c[Austria as usize].ussr = 1;
        state
    }
    pub fn advance_ply(&mut self) -> Option<Side> {
        let win = self.check_win();
        if let Side::US = self.side {
            self.ar += 1;
        }
        self.side = self.side.opposite();
        self.deck.flush_pending();
        win
    }
    pub fn advance_turn(&mut self) -> Option<Side> {
        use std::cmp::max;
        let us_held = self.deck.held_scoring(Side::US);
        let ussr_held = self.deck.held_scoring(Side::USSR);
        // Holding cards is illegal, but it's possible in the physical game
        if us_held && ussr_held {
            return Some(Side::US); // US wins if both players cheat
        } else if us_held {
            return Some(Side::USSR);
        } else if ussr_held {
            return Some(Side::US);
        }
        // Mil ops
        let defcon = self.defcon;
        let us_pen = max(defcon - self.mil_ops[Side::US as usize], 0);
        let ussr_pen = max(defcon - self.mil_ops[Side::USSR as usize], 0);
        // These are penalties, so the signs are reversed from usual
        self.vp -= us_pen;
        self.vp += ussr_pen;
        self.turn += 1;
        // Reset Defcon and Mil ops for next turn
        self.defcon = std::cmp::min(defcon + 1, 5);
        self.mil_ops[0] = 0;
        self.mil_ops[1] = 0;
        // Check win before cleanup due to scoring cards held
        let win = self.check_win();
        self.deck.end_turn_cleanup();
        self.turn_effect_clear();
        win
    }
    pub fn check_win(&self) -> Option<Side> {
        dbg!(self.defcon);
        if self.defcon < 2 {
            return Some(self.side.opposite());
        }
        if self.vp >= 20 {
            return Some(Side::US);
        } else if self.vp <= -20 {
            return Some(Side::USSR);
        }
        None
    }
    pub fn side(&self) -> &Side {
        &self.side
    }
    pub fn max_ar(&self, side: Side) -> i8 {
        if self.turn <= 3 {
            6
        } else if self.space[side as usize] == 8 {
            8
        } else if side == Side::US && self.has_effect(side, Effect::NorthSeaOil) {
            8
        } else {
            7
        }
    }
    pub fn valid_countries(&self) -> &[Country] {
        let len = self.countries.len();
        &self.countries[0..len - 2]
    }
    pub fn apply_restriction(&self, history: &[DecodedChoice], decision: &mut Decision) {
        if let Some(restrict) = &self.restrict {
            match restrict {
                Restriction::Limit(num) => {
                    let counter: Counter<usize> = history
                        .iter()
                        .filter_map(|x| {
                            if x.action == decision.action {
                                x.choice
                            } else {
                                None
                            }
                        })
                        .collect();
                    let bad: HashSet<_> = counter
                        .into_map()
                        .into_iter()
                        .filter_map(|(k, v)| if v >= *num { Some(k) } else { None })
                        .collect();
                    if !bad.is_empty() {
                        let vec: Vec<_> = decision
                            .allowed
                            .slice()
                            .iter()
                            .copied()
                            .filter(|x| !bad.contains(x))
                            .collect();
                        decision.allowed = vec.into();
                    }
                }
            }
        }
    }
    pub fn resolve_action<R: TwilightRand>(
        &mut self,
        mut decision: Decision,
        choice: Option<usize>,
        pending: &mut Vec<Decision>,
        history: &mut Vec<DecodedChoice>,
        rng: &mut R,
    ) -> Option<Decision> {
        let choice = match choice {
            Some(c) => c,
            None => {
                match decision.action {
                    Action::Event => 0,
                    Action::ClearEvent => {
                        self.current_event = None;
                        return None;
                    }
                    _ => return None, // Pass, implicit or explicit
                }
            }
        };
        let side = decision.agent;
        // Check WWBY
        if side == Side::US {
            self.wwby(decision.action == Action::Event && choice == Card::UN_Intervention as usize);
        }
        match decision.action {
            Action::EventOps => {
                let card = Card::from_index(choice);
                let ops = card.modified_ops(decision.agent, self);
                let conduct = Decision::conduct_ops(decision.agent, ops);
                let event = Decision::new_event(side, card);
                pending.push(conduct);
                pending.push(event);
                self.deck.play_card(side, card).expect("Found");
            }
            Action::OpsEvent => {
                let card = Card::from_index(choice);
                let ops = card.modified_ops(decision.agent, self);
                let conduct = Decision::conduct_ops(decision.agent, ops);
                let event = Decision::new_event(side, card);
                pending.push(event);
                pending.push(conduct);
                if card == Card::The_China_Card {
                    self.china = true;
                }
                self.deck.play_card(side, card).expect("Found");
            }
            Action::Ops => {
                let card = Card::from_index(choice);
                let mut ops = card.modified_ops(decision.agent, self);
                if card == Card::The_China_Card {
                    ops += 1;
                    self.china = true;
                }
                let conduct = Decision::conduct_ops(decision.agent, ops);
                pending.push(conduct);
                self.deck.play_card(side, card).expect("Found");
            }
            Action::Event => {
                let card = Card::from_index(choice);
                self.current_event = Some(card);
                self.deck.play_card(side, card).expect("Found");
                if card.event(self, pending, rng) && card.is_starred() {
                    self.deck.remove_card(card).expect("Remove Failure");
                }
            }
            Action::SpecialEvent => {
                let card = self.current_event;
                card.unwrap().special_event(self, choice, pending, rng);
                // Todo reset current event ?
            }
            Action::Space => {
                let card = Card::from_index(choice);
                let roll = rng.roll(decision.agent);
                self.space_card(decision.agent, roll);
                self.discard_card(decision.agent, card);
            }
            Action::Discard => {
                let card = Card::from_index(choice);
                let side = decision.agent; // Todo Aldrich Ames
                                           // Clear Quagmire / Bear Trap if applicable
                if side == Side::US {
                    if let Some(index) = self.effect_pos(side, Effect::Quagmire) {
                        let roll = rng.roll(side);
                        if roll <= 4 {
                            self.clear_effect(side, index);
                        }
                    }
                } else {
                    if let Some(index) = self.effect_pos(side, Effect::BearTrap) {
                        let roll = rng.roll(side);
                        if roll <= 4 {
                            self.clear_effect(side, index);
                        }
                    }
                }
                self.discard_card(side, card);
            }
            Action::Coup => {
                // Todo other free coups
                let free_coup = if let Some(e) = self.current_event {
                    match e {
                        Card::Junta => true,
                        _ => false,
                    }
                } else {
                    false
                };
                let roll = rng.roll(decision.agent);
                let mut ops = decision.quantity;
                if self.china && !Region::Asia.has_country(choice) {
                    ops -= 1;
                    self.china = false;
                }
                self.take_coup(side, choice, ops, roll, free_coup);
                decision.quantity = 1; // Use up all of your ops on one action
            }
            Action::Place => {
                let (q, side) = if let Some(card) = self.current_event {
                    let q = card.influence_quantity(&self, &decision.action, choice);
                    let side = match card.side() {
                        s @ Side::US | s @ Side::USSR => s,
                        Side::Neutral => decision.agent,
                    };
                    (q, side)
                } else {
                    (1, decision.agent)
                };
                for _ in 0..q {
                    self.add_influence(side, choice);
                }
            }
            Action::Influence => {
                let cost = self.add_influence(side, choice);
                if self.china && !Region::Asia.has_country(choice) {
                    decision.quantity -= 1;
                    self.china = false;
                }
                if self.vietnam && !Region::SoutheastAsia.has_country(choice) {
                    decision.quantity -= 1;
                    self.vietnam = false;
                }
                if cost == 2 {
                    decision.quantity -= 1; // Additional 1
                }
            }
            Action::Remove => {
                let event = self.current_event.unwrap();
                let (rs, rq) = event.remove_quantity(side, &self.countries[choice], self.period());
                self.remove_influence(rs, choice, rq);
            }
            Action::War => {
                let brush = match self.current_event.unwrap() {
                    // Todo Brush War
                    _ => false,
                };
                let mut roll = rng.roll(side);
                if brush {
                    roll += 1;
                }
                self.war_target(side, choice, roll);
            }
            Action::Realignment => {
                let (ussr_roll, us_roll) = (rng.roll(Side::USSR), rng.roll(Side::US));
                self.take_realign(choice, us_roll, ussr_roll);
            }
            Action::CubanMissile => {
                self.clear_effect(
                    side,
                    self.effect_pos(side, Effect::CubanMissileCrisis).unwrap(),
                );
                if choice == 0 {
                    self.remove_influence(Side::USSR, CName::Cuba as usize, 2);
                } else if choice == 1 {
                    self.remove_influence(Side::US, CName::WGermany as usize, 2);
                } else {
                    self.remove_influence(Side::US, CName::Turkey as usize, 2);
                }
            }
            Action::RecoverCard => {
                self.deck.recover_card(side, Card::from_index(choice));
            }
            Action::ChooseCard => {
                let event = self.current_event.expect("Some event");
                match event {
                    Card::Missile_Envy => {
                        let chosen_card = Card::from_index(choice);
                        self.deck.play_card(side, chosen_card).expect("Has card");
                        if chosen_card.side() == side {
                            // Opponent Card -> Ops
                            let ops = chosen_card.modified_ops(side.opposite(), self);
                            let dec = Decision::with_quantity(
                                side.opposite(),
                                Action::ConductOps,
                                &[],
                                ops,
                            );
                            pending.push(dec);
                        } else {
                            // ME eventer side card, or neutral
                            let dec = Decision::new_event(side.opposite(), chosen_card);
                            pending.push(dec);
                        }
                        self.add_effect(side, Effect::MissileEnvy);
                    }
                    _ => unimplemented!(),
                }
            }
            Action::ChangeDefcon => self.defcon = choice as i8,
            Action::BeginAr | Action::ConductOps | Action::Pass | Action::ClearEvent => {
                unreachable!()
            }
        }
        let decoded = DecodedChoice::new(decision.action, Some(choice));
        history.push(decoded);
        decision.next_decision(&history, self)
    }
    fn wwby(&mut self, safe: bool) {
        if let Some(pos) = self.effect_pos(Side::US, Effect::WWBY) {
            if !safe {
                self.vp -= 3;
            }
            self.us_effects.swap_remove(pos);
        }
    }
    /// Return true if the side has the effect, else false.
    pub fn has_effect(&self, side: Side, effect: Effect) -> bool {
        let vec = match side {
            Side::US => &self.us_effects,
            Side::USSR => &self.ussr_effects,
            _ => unimplemented!(),
        };
        vec.iter().any(|e| *e == effect)
    }
    /// Returns the index of the effect if it is in play, or else None.
    pub fn effect_pos(&self, side: Side, effect: Effect) -> Option<usize> {
        let vec = match side {
            Side::US => &self.us_effects,
            Side::USSR => &self.ussr_effects,
            _ => unimplemented!(),
        };
        vec.iter().position(|e| *e == effect)
    }
    pub fn add_effect(&mut self, side: Side, effect: Effect) {
        let vec = match side {
            Side::US => &mut self.us_effects,
            Side::USSR => &mut self.ussr_effects,
            _ => unimplemented!(),
        };
        vec.push(effect);
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
    pub fn remove_influence(&mut self, side: Side, country_index: usize, num: i8) {
        let c = &mut self.countries[country_index];
        // Require checking for influence prior to this step
        match side {
            Side::US => c.us -= num,
            Side::USSR => c.ussr -= num,
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
    pub fn remove_all<T: Into<usize>>(&mut self, side: Side, country: T) {
        let c = &mut self.countries[country.into()];
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
    /// Clears all effects that are meant to only last for a single turn.
    pub fn turn_effect_clear(&mut self) {
        self.us_effects = self
            .us_effects
            .iter()
            .copied()
            .filter(|e| e.permanent())
            .collect();
        self.ussr_effects = self
            .ussr_effects
            .iter()
            .copied()
            .filter(|e| e.permanent())
            .collect();
    }
    /// Returns which period of the war the game is in
    pub fn period(&self) -> Period {
        if self.turn <= 3 {
            Period::Early
        } else if self.turn <= 7 {
            Period::Middle
        } else {
            Period::Late
        }
    }
    /// Calculates the number of adjacent controlled countries for use in wars.
    fn adjacent_controlled(&self, country_index: usize, side: Side) -> i8 {
        EDGES[country_index].iter().fold(0, |acc, c| {
            if self.countries[*c].controller() == side {
                acc + 1
            } else {
                acc
            }
        })
    }
    /// Calculates a war on a standard 4-6 roll. Modify Brush War +1, Israel -1
    /// for these respective events
    pub fn war_target(&mut self, war_side: Side, country_index: usize, mut roll: i8) -> bool {
        let adjacent = self.adjacent_controlled(country_index, war_side.opposite());
        roll -= adjacent;
        if roll >= 4 {
            self.war_flip(war_side, country_index);
            true
        } else {
            false
        }
    }
    fn war_flip(&mut self, war_side: Side, country_index: usize) {
        let c = &mut self.countries[country_index];
        match war_side {
            Side::US => {
                let opp = c.ussr;
                c.ussr = 0;
                c.us += opp;
            }
            Side::USSR => {
                let opp = c.us;
                c.us = 0;
                c.ussr += opp;
            }
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn add_mil_ops(&mut self, side: Side, amount: i8) {
        let mil_ops = self.mil_ops[side as usize];
        self.mil_ops[side as usize] = std::cmp::min(5, mil_ops + amount);
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
    pub fn discard_card(&mut self, side: Side, card: Card) {
        self.deck.play_card(side, card).expect("Found");
    }
    /// Returns cards in hand at least the given value. China is never
    /// included.
    pub fn cards_at_least(&self, side: Side, val: i8) -> Vec<Card> {
        let cards = self.deck.hand(side);
        let offset = self.base_ops_offset(side);
        cards
            .iter()
            .copied()
            .filter(|c| c.ops(offset) >= val)
            .collect()
    }
    /// Calculates the base offset to card ops, as influenced by Containment,
    /// Brezhnev, and RSP.
    pub fn base_ops_offset(&self, side: Side) -> i8 {
        let mut offset = 0;
        match side {
            Side::US => {
                if self.ussr_effects.contains(&Effect::RedScarePurge) {
                    offset -= 1;
                }
                if self.us_effects.contains(&Effect::Containment) {
                    offset += 1;
                }
            }
            Side::USSR => {
                if self.us_effects.contains(&Effect::RedScarePurge) {
                    offset -= 1;
                }
                if self.ussr_effects.contains(&Effect::Brezhnev) {
                    offset += 1;
                }
            }
            _ => unimplemented!(),
        }
        offset
    }
    pub fn can_space(&self, side: Side, ops: i8) -> bool {
        let me = side as usize;
        let opp = side.opposite() as usize;
        let my_space = self.space[me];
        if my_space >= 8 {
            return false; // Space race completed!
        }
        let space_allowed = self.space_attempts[me] < 1
            || self.space_attempts[me] < 2 && my_space >= 2 && self.space[opp] < 2;
        if my_space <= 3 {
            space_allowed && ops >= 2
        } else if my_space <= 6 {
            space_allowed && ops >= 3
        } else {
            space_allowed && ops >= 4
        }
    }
    pub fn space_card(&mut self, side: Side, roll: i8) -> bool {
        let me = side as usize;
        let opp = side.opposite() as usize;
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
    pub fn legal_space(&self, side: Side) -> Vec<usize> {
        let mut vec = Vec::new();
        let hand = self.deck.hand(side);
        let ops_offset = self.base_ops_offset(side);
        for &c in hand.iter() {
            if self.can_space(self.side, c.ops(ops_offset)) {
                vec.push(c as usize);
            }
        }
        if self.deck.china_available(side) {
            let china = Card::The_China_Card;
            if self.can_space(side, china.base_ops() + ops_offset) {
                vec.push(china as usize) // Legal, if not advisable
            }
        }
        vec
    }
    pub fn legal_coup_realign(&self, side: Side) -> Vec<usize> {
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
            if side == Side::USSR && self.has_effect(Side::US, Effect::Nato) {
                let mut set: HashSet<usize> = EUROPE
                    .iter()
                    .filter_map(|&x| {
                        let c = &self.countries[x];
                        if c.has_influence(opp) && c.controller() != Side::US {
                            Some(x)
                        } else {
                            None
                        }
                    })
                    .collect();
                // Todo other NATO exceptions
                let france = &self.countries[CName::France as usize];
                if france.controller() == Side::US {
                    if self.has_effect(Side::USSR, Effect::DeGaulle) {
                        set.insert(CName::France as usize);
                    }
                }
                vec.extend(set.iter());
            } else {
                vec.extend(valid(&EUROPE));
            }
        }
        vec
    }
    pub fn legal_influence(&self, agent: Side, ops: i8) -> Vec<usize> {
        let china = self.china;
        let vietnam = self.vietnam;
        let real_ops = ops - (china as i8) - (vietnam as i8);
        let a = access(self, agent);
        if real_ops > 1 {
            // Doesn't need to tap into bonus influence
            a
        } else if ops <= 1 {
            // Cannot break control anywhere
            assert!(ops > 0);
            a.into_iter()
                .filter(|x| {
                    let c = &self.countries[*x];
                    c.controller() != agent.opposite()
                })
                .collect()
        } else if china {
            // If China is in play, vietnam revolts is irrelevant for legality
            // since no action costs more than 2 ops and Southeast Asia
            // is a subset of Asia
            if ops >= 2 {
                // Can break across Asia, but cannot elsewhere
                a.into_iter()
                    .filter(|x| {
                        let c = &self.countries[*x];
                        c.controller() != agent.opposite() || Region::Asia.has_country(*x)
                    })
                    .collect()
            } else {
                // Can only place in uncontrolled Asia
                a.into_iter()
                    .filter(|x| {
                        let c = &self.countries[*x];
                        c.controller() != agent.opposite() && Region::Asia.has_country(*x)
                    })
                    .collect()
            }
        } else if vietnam {
            if ops >= 2 {
                // Can break in SE Asia, but cannot elsewhere
                a.into_iter()
                    .filter(|x| {
                        let c = &self.countries[*x];
                        c.controller() != agent.opposite() || Region::SoutheastAsia.has_country(*x)
                    })
                    .collect()
            } else {
                // Can only place in uncontrolled SE Asia
                a.into_iter()
                    .filter(|x| {
                        let c = &self.countries[*x];
                        c.controller() != agent.opposite() && Region::SoutheastAsia.has_country(*x)
                    })
                    .collect()
            }
        } else {
            unreachable!() // Todo figure out if this is actually unreachable
        }
    }
    pub fn legal_war(&self, side: Side) -> Allowed {
        if side == Side::USSR && self.has_effect(Side::US, Effect::Nato) {
            let vec: Vec<_> = BRUSH_TARGETS
                .iter()
                .copied()
                .filter(|&i| {
                    !(Region::Europe.has_country(i) && self.countries[i].controller() == Side::US)
                })
                .collect();
            Allowed::new_owned(vec)
        } else {
            Allowed::new_slice(&BRUSH_TARGETS[..])
        }
    }
    pub fn legal_cuban(&self, side: Side) -> Allowed {
        match side {
            Side::USSR => {
                if self.countries[CName::Cuba as usize].ussr >= 2 {
                    Allowed::new_owned(vec![0])
                } else {
                    Allowed::new_empty()
                }
            }
            Side::US => {
                let mut vec = Vec::new();
                if self.countries[CName::WGermany as usize].us >= 2 {
                    vec.push(1);
                }
                if self.countries[CName::Turkey as usize].us >= 2 {
                    vec.push(2);
                }
                if vec.is_empty() {
                    Allowed::new_empty()
                } else {
                    Allowed::new_owned(vec)
                }
            }
            _ => unimplemented!(),
        }
    }
    pub fn mil_ops(&self, side: Side) -> i8 {
        self.mil_ops[side as usize]
    }
    pub fn set_limit(&mut self, limit: usize, pending_actions: &mut Vec<Decision>) {
        self.restrict = Some(Restriction::Limit(limit));
        // Todo restriction clear more nicely
    }
    pub fn ar_left(&self, side: Side) -> i8 {
        let goal = match self.period() {
            Period::Early => 6,
            _ => 7,
        };
        match side {
            Side::US => goal - self.ar + 1,
            Side::USSR => {
                if self.side == Side::US {
                    goal - self.ar
                } else {
                    goal - self.ar + 1
                }
            }
            _ => unimplemented!(),
        }
    }
    pub fn is_final_scoring(&self) -> bool {
        self.turn > 10
    }
}

#[derive(Clone, Copy)]
pub enum Period {
    Early,
    Middle,
    Late,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{RandAgent, ScriptedAgent};
    use crate::country;
    use crate::game::Game;
    use crate::tensor::OutputIndex;
    #[test]
    fn test_influence() {
        // let mut state = GameState::four_four_two();
        // state.control(Side::USSR, CName::Vietnam);
        // state.control(Side::US, CName::Thailand);
        // // state.add_effect(Side::USSR, Effect::VietnamRevolts);
        // // Todo determine this better
        // state.china = true;
        // state.vietnam = true;
        // state.current_event = Some(Card::The_China_Card);
        // state.ar = 1;
        // state.turn = 1;
        // let test: Vec<_> = country::access(&state, Side::USSR).into_iter().map(|x| {
        //     CName::from_index(x)
        // }).collect();
        // // Todo determine ops automatically
        // let d = Decision::determine(Side::USSR, Action::Influence, 6, &state);
        // let laos = influence_in(CName::LaosCambodia);
        // let thai = influence_in(CName::Thailand); // US Controlled
        // let afghan = influence_in(CName::Afghanistan);
        // let poland = influence_in(CName::Poland);
        // let okay = vec![
        //     vec![laos, laos, laos, thai, laos],
        //     vec![afghan, afghan, afghan, laos, laos],
        //     vec![poland, laos, poland, laos],
        // ];
        // let nok = vec![
        //     vec![laos, laos, laos, laos, laos, thai],
        //     vec![poland, laos, laos, laos, laos],
        //     vec![afghan, afghan, afghan, afghan, thai],
        // ];
        // let rng = DebugRand::new_empty();
        // for x in okay {
        //     let mut s = state.clone();
        //     let us_agent = ScriptedAgent::new(x);
        //     let mut game = Game::new(agent, RandAgent::new(), s, rng.clone());
        //     agent.legal_line(&mut game, 1, 1);
        // }
        // for y in nok {
        //     let mut s = state.clone();
        //     let agent = ScriptedAgent::new(y);
        //     assert!(!agent.legal_line(&mut s, vec![d.clone()], rng.clone()))
        // }
    }
    fn influence_in(country: CName) -> OutputIndex {
        OutputIndex::new(Action::Influence.offset() + country as usize)
    }
    #[test]
    fn count_actions() {
        // let mut state = GameState::new();
        // let cards = &[Card::Duck_and_Cover, Card::Arab_Israeli_War, Card::Blockade];
        // let sizes = &[7, 5, 4];
        // for (&c, &s) in cards.into_iter().zip(sizes.into_iter()) {
        //     // Todo allow for play where we only simulate one side
        //     state.deck.ussr_hand_mut().push(c);
        //     let mut pending = Vec::new();
        //     state.use_card(c, &mut pending);
        //     let x = &pending.pop().unwrap();
        //     // match &x.action {
        //     //     Action::AfterStates(vec) => assert_eq!(vec.len(), s),
        //     //     _ => assert!(false),
        //     // }
        // }
    }
}
