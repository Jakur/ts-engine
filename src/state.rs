use crate::action::{Action, Decision, Restriction, EventTime};
use crate::agent::{Actors, Agent};
use crate::card::*;
use crate::country::*;

use counter::Counter;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use std::collections::HashSet;

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
    pub rng: SmallRng,
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
            rng: SmallRng::from_entropy(),
            deck: Deck::new(),
            restrict: None,
            current_event: None,
            vietnam: false,
            china: false,
        }
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
    pub fn valid_countries(&self) -> &[Country] {
        let len = self.countries.len();
        &self.countries[0..len - 2]
    }
    pub fn roll(&mut self) -> i8 {
        self.rng.gen_range(1, 7)
    }
    pub fn choose_card<A: Agent, B: Agent>(
        &self,
        actors: &Actors<A, B>,
        can_pass: bool,
    ) -> Option<Card> {
        let agent = actors.get(self.side);
        let hand = self.deck.hand(self.side);
        let china = self.deck.china_available(self.side);
        let (card, _eval) = agent.decide_card(&self, &hand[..], china, true, can_pass);
        card
    }
    pub fn use_card(&mut self, card: Card, pending_actions: &mut Vec<Decision>) {
        unimplemented!()
    }
    pub fn card_uses(&self) -> Vec<Action> {
        unimplemented!()
    }
    pub fn set_event(&mut self, card: Card) {
        self.current_event = Some(card);
    }
    /// Returns the standard allowed actions if they differ from the decision
    /// slice, or else None.
    pub fn standard_allowed(
        &self,
        dec: &Decision,
        history: &[usize],
    ) -> Option<Vec<usize>> {
        let Decision {
            agent,
            action,
            allowed,
            quantity,
        } = dec;
        let allowed = allowed.slice();
        let mut vec = match action {
            Action::StandardOps => {
                let china = self.china;
                let vietnam = self.vietnam;
                let real_ops = dec.quantity - (china as i8) - (vietnam as i8);
                let a = access(self, *agent);
                if real_ops > 1 { // Doesn't need to tap into bonus influence
                    Some(a)
                } else if *quantity <= 1 { // Cannot break control anywhere
                    assert!(*quantity > 0);
                    Some(a.into_iter().filter(|x| {
                        let c = &self.countries[*x];
                        c.controller() != agent.opposite()
                    }).collect())
                } else if china {
                    // If China is in play, vietnam revolts is irrelevant for legality
                    // since no action costs more than 2 ops and Southeast Asia
                    // is a subset of Asia
                    if *quantity >= 2 {
                        // Can break across Asia, but cannot elsewhere
                        Some(a.into_iter().filter(|x| {
                            let c = &self.countries[*x];
                            c.controller() != agent.opposite() || Region::Asia.has_country(*x)
                        }).collect())
                    } else {
                        // Can only place in uncontrolled Asia
                        Some(a.into_iter().filter(|x| {
                            let c = &self.countries[*x];
                            c.controller() != agent.opposite() && Region::Asia.has_country(*x)
                        }).collect())
                    }
                } else if vietnam {
                    if *quantity >= 2 {
                        // Can break in SE Asia, but cannot elsewhere
                        Some(a.into_iter().filter(|x| {
                            let c = &self.countries[*x];
                            c.controller() != agent.opposite() || Region::SoutheastAsia.has_country(*x)
                        }).collect())
                    } else {
                        // Can only place in uncontrolled SE Asia
                        Some(a.into_iter().filter(|x| {
                            let c = &self.countries[*x];
                            c.controller() != agent.opposite() && Region::SoutheastAsia.has_country(*x)
                        }).collect())
                    }
                } else {
                    unreachable!() // Todo figure out if this is actually unreachable
                }
            },
            Action::Coup | Action::Realignment => {
                let opp = agent.opposite();
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
                    if dec.agent == Side::USSR && self.has_effect(Side::US, Effect::Nato).is_some()
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
                            if self.has_effect(Side::USSR, Effect::DeGaulle).is_some() {
                                set.insert(CName::France as usize);
                            }
                        }
                        vec.extend(set.iter());
                    } else {
                        vec.extend(valid(&EUROPE));
                    }
                }
                Some(vec)
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
                if side == Side::USSR && brush && self.has_effect(Side::US, Effect::Nato).is_some() {
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
                        if vec.is_some() {
                            vec = Some(
                                vec.unwrap()
                                    .into_iter()
                                    .filter(|x| !bad.contains(x))
                                    .collect(),
                            );
                        } else {
                            vec = Some(
                                allowed
                                    .iter()
                                    .filter_map(|x| if bad.contains(x) { None } else { Some(*x) })
                                    .collect(),
                            );
                        }
                    }
                }
            }
        }
        vec
    }
    pub fn resolve_card(&mut self, decision: Decision, card: Card) {
        match decision.action {
            Action::Space => {
                let roll = self.roll();
                self.space_card(decision.agent, roll);
                self.discard_card(decision.agent, card);
            },
            Action::Discard => {
                let side = decision.agent; // Todo Aldrich Ames
                self.discard_card(side, card);
            }
            _ => unimplemented!(),
        }
    }
    pub fn resolve_action(
        &mut self,
        mut decision: Decision,
        choice: Option<usize>,
        pending: &mut Vec<Decision>,
        history: &mut Vec<usize>,
    ) {
        let choice = if choice.is_some() {
            choice.unwrap()
        } else {
            return; // Pass?
        };
        let side = decision.agent;
        let ops = decision.quantity; // Todo figure out if this is wrong
        match decision.action {
            Action::Event => {
                let card = self.current_event;
                card.unwrap().event(self, choice, pending);
                // Todo reset current event ? 
            }
            Action::Coup => {
                let free_coup = false; // Todo free coup
                let roll = self.roll(); // Todo more flexible entropy source
                self.take_coup(side, choice, ops, roll, free_coup);
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
                let mut roll = self.roll();
                if brush {
                    roll += 1;
                }
                self.war_target(side, choice, roll);
            }
            Action::Realignment => {
                let (us_roll, ussr_roll) = (self.roll(), self.roll());
                self.take_realign(choice, us_roll, ussr_roll);
            }
            _ => todo!(),
        }
        decision.quantity -= 1;
        if decision.quantity > 0 {
            pending.push(decision);
        }
        history.push(choice);
    }
    /// Returns the index of the effect if it is in play, or else None
    pub fn has_effect(&self, side: Side, effect: Effect) -> Option<usize> {
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
    /// Returns hand indices for cards at least the given value. China is never
    /// included.
    pub fn cards_at_least(&self, side: Side, val: i8) -> Vec<usize> {
        let cards = self.deck.hand(side);
        let offset = self.base_ops_offset(side);
        cards
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                let mut x = c.base_ops() + offset;
                if x < 1 {
                    x = 1;
                } else if x > 4 {
                    x = 4;
                }
                if x >= val {
                    Some(i)
                } else {
                    None
                }
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
    pub fn resolve_actions<A: Agent, B: Agent>(&mut self, actors: &Actors<A, B>, pending: Vec<Decision>) {
        unimplemented!();
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
    #[test]
    fn basic_actions() {
        use crate::agent;
        let template = GameState::new();
        let agent = agent::DebugAgent::new(
            Action::Coup(0, false),
            Card::De_Stalinization,
            &[CName::UK as usize, CName::Canada as usize],
        );
        let agents = Actors::new(agent.clone(), agent.clone());
        let mut a = template.clone();
        let mut pending = Vec::new();
        Card::Socialist_Governments.event(&mut a, 0, &mut pending);
        a.resolve_actions(&agents, pending);
        assert_eq!(a.countries[CName::UK as usize].us, 3);
        assert_eq!(a.countries[CName::Canada as usize].us, 1);
        a.defcon = 2;
        for (x, y) in [(Side::US, 0), (Side::USSR, 2)].into_iter() {
            let allowed = a.standard_allowed(
                &Decision::new_no_allowed(*x, Action::Coup(1, false)),
                &[],
                &vec![],
            );
            assert_eq!(*y, allowed.unwrap().len());
        }
    }
    #[test]
    fn count_actions() {
        let mut state = GameState::new();
        let cards = &[Card::Duck_and_Cover, Card::Arab_Israeli_War, Card::Blockade];
        let sizes = &[7, 5, 4];
        for (&c, &s) in cards.into_iter().zip(sizes.into_iter()) {
            // Todo allow for play where we only simulate one side
            state.deck.ussr_hand_mut().push(c);
            let mut pending = Vec::new();
            state.use_card(c, &mut pending);
            let x = &pending.pop().unwrap();
            // match &x.action {
            //     Action::AfterStates(vec) => assert_eq!(vec.len(), s),
            //     _ => assert!(false),
            // }
        }
    }
}
