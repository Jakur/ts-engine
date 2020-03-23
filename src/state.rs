use crate::action::{Action, Allowed, Decision, Restriction, EventTime};
use crate::card::*;
use crate::country::*;

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
            ar: 1,
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
        win
    }
    pub fn check_win(&self) -> Option<Side> {
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
    pub fn set_event(&mut self, card: Card) {
        self.current_event = Some(card);
    }
    /// Returns the standard allowed actions if they differ from the decision
    /// slice, or else None.
    pub fn standard_allowed(
        &self,
        dec: &mut Decision,
        history: &[usize],
    ) {
        let Decision {
            agent,
            action,
            allowed: _,
            quantity,
        } = dec;
        let vec = match action {
            Action::StandardOps => {
                Some(self.legal_influence(*agent, *quantity))
            },
            Action::Coup | Action::Realignment => {
                Some(self.legal_coup_realign(*agent))
            }
            Action::Remove => {
                // Destal is the only case where you remove your own influence
                let side = match self.current_event {
                    Some(e) if e == Card::De_Stalinization => {
                        Side::USSR
                    }
                    _ => agent.opposite()
                };
                Some(dec.allowed.slice().iter().copied().filter(|x| {
                    self.countries[*x].has_influence(side)
                }).collect())
            }
            Action::War => {
                let (side, brush) = match self.current_event.unwrap() {
                    // Todo Brush War
                    _ => (Side::USSR, false)
                };
                if side == Side::USSR && brush && self.has_effect(Side::US, Effect::Nato) {
                    Some(
                        BRUSH_TARGETS
                            .iter()
                            .copied()
                            .filter(|&i| {
                                !(Region::Europe.has_country(i)
                                    && self.countries[i].controller() == Side::US)
                            })
                            .collect(),
                    )
                } else {
                    None // Default
                }
            }
            _ => None,
        };
        if let Some(v) = vec {
            dec.allowed = v.into();
        }
        // Avoid restrictions in special cases
        if let Some(e) = self.current_event {
            if e == Card::De_Stalinization && *action == Action::Remove {
                return
            }
        }
        // Todo figure out if restriction is always limit
        self.apply_restriction(history, dec);
    }
    fn apply_restriction(&self, history: &[usize], decision: &mut Decision) {
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
                        let vec: Vec<_> = decision.allowed.slice().iter().copied()
                            .filter(|x| !bad.contains(x)).collect();
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
        history: &mut Vec<usize>,
        rng: &mut R,
    ) {
        let choice = if choice.is_some() {
            choice.unwrap()
        } else {
            match decision.action {
                Action::Event => 0,
                Action::Pass => return,
                _ => unimplemented!(),
            }
        };
        let side = decision.agent;
        match decision.action {
            Action::PlayCard => {
                let (card, time) = Action::play_card_data(choice);
                let ops = card.modified_ops(decision.agent, self);
                let conduct = Decision::conduct_ops(decision.agent, ops);
                match time {
                    EventTime::After => {
                        let event = Decision::new_event(card);
                        pending.push(event);
                        pending.push(conduct);
                    }
                    EventTime::Before => {
                        let event = Decision::new_event(card);
                        pending.push(conduct);
                        pending.push(event);
                    }
                    EventTime::Never => pending.push(conduct),
                }
            }
            Action::Space => {
                let card = Card::from_index(choice);
                let roll = rng.roll();
                self.space_card(decision.agent, roll);
                self.discard_card(decision.agent, card);
            },
            Action::Discard => {
                let card = Card::from_index(choice);
                let side = decision.agent; // Todo Aldrich Ames
                // Clear Quagmire / Bear Trap if applicable
                if side == Side::US {
                    if let Some(index) = self.effect_pos(side, Effect::Quagmire) {
                        let roll = rng.roll();
                        if roll <= 4 {
                            self.clear_effect(side, index);
                        }
                    }
                } else {
                    if let Some(index) = self.effect_pos(side, Effect::BearTrap) {
                        let roll = rng.roll();
                        if roll <= 4 {
                            self.clear_effect(side, index);
                        }
                    }
                }
                self.discard_card(side, card);
            },
            Action::Event => {
                let card = self.current_event;
                card.unwrap().event(self, pending, rng);
            }
            Action::SpecialEvent => {
                let card = self.current_event;
                card.unwrap().special_event(self, choice, pending, rng);
                // Todo reset current event ? 
            }
            Action::Coup => {
                let free_coup = false; // Todo free coup
                let roll = rng.roll(); // Todo more flexible entropy source
                self.take_coup(side, choice, decision.quantity, roll, free_coup);
            }
            Action::Place => {
                let c = self.current_event.expect("Only place through event");
                let q = c.influence_quantity(&self, &decision.action, choice);
                let side = match c.side() {
                    s @ Side::US | s @ Side::USSR => s,
                    Side::Neutral => decision.agent,
                };
                for _ in 0..q {
                    self.add_influence(side, choice);
                }
            }
            Action::StandardOps => {
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
                let (s, q) = self.current_event.unwrap()
                    .remove_quantity(decision.agent, &self.countries[choice], self.period());
                self.remove_influence(s, choice, q);
            }
            Action::War => {
                let brush = match self.current_event.unwrap() {
                    // Todo Brush War
                    _ => false,
                };
                let mut roll = rng.roll();
                if brush {
                    roll += 1;
                }
                self.war_target(side, choice, roll);
            }
            Action::Realignment => {
                let (ussr_roll, us_roll) = (rng.roll(), rng.roll());
                self.take_realign(choice, us_roll, ussr_roll);
            },
            Action::CubanMissile => {
                self.clear_effect(side, self.effect_pos(side, Effect::CubanMissileCrisis).unwrap());
                if choice == 0 {
                    self.remove_influence(Side::USSR, CName::Cuba as usize, 2);
                } else if choice == 1 {
                    self.remove_influence(Side::US, CName::WGermany as usize, 2);
                } else {
                    self.remove_influence(Side::US, CName::Turkey as usize, 2);          
                }
            },
            Action::RecoverCard => {
                self.deck.recover_card(side, Card::from_index(choice));
            },
            Action::ChangeDefcon => self.defcon = choice as i8,
            Action::BeginAr | Action::ConductOps | Action::Pass => unimplemented!(),
        }
        decision.quantity -= 1;
        if let Action::StandardOps = decision.action {
            if let Some(d) = decision.next_influence(self) {
                pending.push(d);
            }
        } else {
            if decision.quantity > 0 {
                pending.push(decision);
            }
        }
        history.push(choice);
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
        self.mil_ops[side as usize] += amount;
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
        self.deck.play_card(side, card);
    }
    /// Returns cards in hand at least the given value. China is never
    /// included.
    pub fn cards_at_least(&self, side: Side, val: i8) -> Vec<Card> {
        let cards = self.deck.hand(side);
        let offset = self.base_ops_offset(side);
        cards
            .iter()
            .copied()
            .filter(|c| {
                c.ops(offset) >= val
            })
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
            if self.can_space(self.side, c.base_ops() + ops_offset) {
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
            if side == Side::USSR && self.has_effect(Side::US, Effect::Nato)
            {
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
        if real_ops > 1 { // Doesn't need to tap into bonus influence
            a
        } else if ops <= 1 { // Cannot break control anywhere
            assert!(ops > 0);
            a.into_iter().filter(|x| {
                let c = &self.countries[*x];
                c.controller() != agent.opposite()
            }).collect()
        } else if china {
            // If China is in play, vietnam revolts is irrelevant for legality
            // since no action costs more than 2 ops and Southeast Asia
            // is a subset of Asia
            if ops >= 2 {
                // Can break across Asia, but cannot elsewhere
                a.into_iter().filter(|x| {
                    let c = &self.countries[*x];
                    c.controller() != agent.opposite() || Region::Asia.has_country(*x)
                }).collect()
            } else {
                // Can only place in uncontrolled Asia
                a.into_iter().filter(|x| {
                    let c = &self.countries[*x];
                    c.controller() != agent.opposite() && Region::Asia.has_country(*x)
                }).collect()
            }
        } else if vietnam {
            if ops >= 2 {
                // Can break in SE Asia, but cannot elsewhere
                a.into_iter().filter(|x| {
                    let c = &self.countries[*x];
                    c.controller() != agent.opposite() || Region::SoutheastAsia.has_country(*x)
                }).collect()
            } else {
                // Can only place in uncontrolled SE Asia
                a.into_iter().filter(|x| {
                    let c = &self.countries[*x];
                    c.controller() != agent.opposite() && Region::SoutheastAsia.has_country(*x)
                }).collect()
            }
        } else {
            unreachable!() // Todo figure out if this is actually unreachable
        }
    }
    pub fn legal_special_event(&self, side: Side) -> Vec<Card> {
        let hand = self.deck.hand(side);
        hand.iter().copied().filter(|x| {
            x.max_e_choices() > 1
        }).collect()
    }
    pub fn legal_war(&self, side: Side) -> Allowed {
        if side == Side::USSR && self.has_effect(Side::US, Effect::Nato) {
            let vec: Vec<_> = BRUSH_TARGETS
                    .iter()
                    .copied()
                    .filter(|&i| {
                        !(Region::Europe.has_country(i)
                            && self.countries[i].controller() == Side::US)
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
            },
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
            },
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
    use crate::tensor::OutputIndex;
    use crate::agent::DebugAgent;
    use crate::country;
    #[test]
    fn test_influence() {
        let mut state = GameState::four_four_two();
        state.control(Side::USSR, CName::Vietnam);
        state.control(Side::US, CName::Thailand);
        // state.add_effect(Side::USSR, Effect::VietnamRevolts);
        // Todo determine this better
        state.china = true;
        state.vietnam = true;
        state.current_event = Some(Card::The_China_Card);
        let test: Vec<_> = country::access(&state, Side::USSR).into_iter().map(|x| {
            CName::from_index(x)
        }).collect();
        dbg!(test);
        // Todo determine ops automatically
        let d = Decision::determine(Side::USSR, Action::StandardOps, 6, &state);
        let laos = influence_in(CName::LaosCambodia);
        let thai = influence_in(CName::Thailand); // US Controlled
        let afghan = influence_in(CName::Afghanistan);
        let poland = influence_in(CName::Poland);
        let okay = vec![
            vec![laos, laos, laos, thai, laos],
            vec![afghan, afghan, afghan, laos, laos],
            vec![poland, laos, poland, laos],
        ];
        let nok = vec![
            vec![laos, laos, laos, laos, laos, thai],
            vec![poland, laos, laos, laos, laos],
            vec![afghan, afghan, afghan, afghan, thai],
        ];
        let rng = DebugRand::new_empty();
        for x in okay {
            let mut s = state.clone();
            let agent = DebugAgent::new(x);
            assert!(agent.legal_line(&mut s, vec![d.clone()], rng.clone()));
        }
        for y in nok {
            let mut s = state.clone();
            let agent = DebugAgent::new(y);
            assert!(!agent.legal_line(&mut s, vec![d.clone()], rng.clone()))
        }
    }
    fn influence_in(country: CName) -> OutputIndex {
        OutputIndex::new(Action::StandardOps.offset() + country as usize)
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
