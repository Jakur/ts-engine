#![allow(non_camel_case_types)]

use crate::action::{Action, Decision};
use crate::country::{self, CName, Region, Side};
use crate::state::GameState;

use rand::seq::SliceRandom;
use rand::thread_rng;

lazy_static! {
    static ref ATT: Vec<Attributes> = init_cards();
}

#[derive(Clone)]
pub struct Deck {
    us_hand: Vec<Card>,
    ussr_hand: Vec<Card>,
    discard_pile: Vec<Card>,
    draw_pile: Vec<Card>,
    removed: Vec<Card>,
    china: Side,
    china_up: bool,
}

impl Deck {
    pub fn new() -> Self {
        Deck {
            us_hand: Vec::new(),
            ussr_hand: Vec::new(),
            discard_pile: Vec::new(),
            draw_pile: Vec::new(),
            removed: Vec::new(),
            china: Side::USSR,
            china_up: true,
        }
    }
    pub fn hand(&self, side: Side) -> &Vec<Card> {
        match side {
            Side::US => &self.us_hand,
            Side::USSR => &self.ussr_hand,
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn draw_cards(&mut self, target: usize) {
        let mut pick_ussr = true;
        // Oscillating is relevant when reshuffles do occur
        while self.ussr_hand.len() < target && self.us_hand.len() < target {
            let next_card = self.draw_card();
            if pick_ussr {
                self.ussr_hand.push(next_card);
            } else {
                self.us_hand.push(next_card);
            }
            pick_ussr = !pick_ussr;
        }
        while self.ussr_hand.len() < target {
            let c = self.draw_card();
            self.ussr_hand.push(c);
        }
        while self.us_hand.len() < target {
            let c = self.draw_card();
            self.us_hand.push(c);
        }
    }
    /// Searches the discard pile for a played card and removes it.
    pub fn remove_card(&mut self, card: Card) -> bool {
        let found = self.discard_pile.iter().rposition(|&c| c == card);
        if let Some(i) = found {
            let c = self.discard_pile.remove(i); // Should be fast since i should be near the end
            self.removed.push(c);
            true
        } else {
            false
        }
    }
    /// Draws the next card from the draw pile, reshuffling if necessary.
    fn draw_card(&mut self) -> Card {
        match self.draw_pile.pop() {
            Some(c) => c,
            None => {
                self.reshuffle();
                self.draw_card()
            }
        }
    }
    /// Removes and returns the card from the given index.
    /// # Panics
    /// If the index is out of bounds for the hand
    pub fn grab_card(&mut self, side: Side, index: usize) -> Card {
        match side {
            Side::US => self.us_hand.swap_remove(index),
            Side::USSR => self.ussr_hand.swap_remove(index),
            _ => unimplemented!(),
        }
    }
    pub fn us_hand(&self) -> &Vec<Card> {
        &self.us_hand
    }
    pub fn ussr_hand(&self) -> &Vec<Card> {
        &self.ussr_hand
    }
    pub fn discard_pile(&self) -> &Vec<Card> {
        &self.discard_pile
    }
    pub fn draw_pile(&self) -> &Vec<Card> {
        &self.draw_pile
    }
    pub fn removed(&self) -> &Vec<Card> {
        &self.removed
    }
    pub fn play_china(&mut self) {
        self.china = self.china.opposite();
        self.china_up = false;
    }
    pub fn play_card(&mut self, side: Side, index: usize, evented: bool) {
        let hand = match side {
            Side::US => &mut self.us_hand,
            Side::USSR => &mut self.ussr_hand,
            Side::Neutral => unimplemented!(),
        };
        let card = hand.swap_remove(index);
        if evented && card.att().starred {
            self.removed.push(card);
        } else {
            self.discard_pile.push(card);
        }
    }
    pub fn china(&self) -> Side {
        self.china
    }
    fn reshuffle(&mut self) {
        let mut rng = thread_rng();
        self.discard_pile.shuffle(&mut rng);
        self.draw_pile.append(&mut self.discard_pile);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Effect {
    ShuttleDiplomacy,
    FormosanResolution,
    IronLady,
    VietnamRevolts,
    RedScarePurge,
    Containment,
    Brezhnev,
    CampDavid,
    AllowNato,
    DeGaulle,
    Nato,
    US_Hand_Revealed,
    USSR_Hand_Revealed,
    US_Japan,
}

pub struct Attributes {
    pub side: Side,
    pub ops: i8,
    pub starred: bool,
    pub scoring: bool,
}

impl Attributes {
    fn new(side: Side, ops: i8) -> Attributes {
        Attributes {
            side,
            ops,
            starred: false,
            scoring: false,
        }
    }
    fn star(mut self) -> Attributes {
        self.starred = true;
        self
    }
    fn scoring(mut self) -> Attributes {
        self.scoring = true;
        self
    }
}

fn init_cards() -> Vec<Attributes> {
    use Side::*;
    let c = Attributes::new;
    let x = vec![
        c(Neutral, 0), // Dummy
        c(Neutral, 0).scoring(),
        c(Neutral, 0).scoring(),
        c(Neutral, 0).scoring(),
        c(US, 3),
        c(US, 3),
        c(Neutral, 4), // China
        c(USSR, 3),
        c(USSR, 2).star(),
        c(USSR, 2).star(),
        c(USSR, 1).star(),
        c(USSR, 2).star(),
        c(USSR, 1).star(),
        c(USSR, 2),
        c(USSR, 3).star(),
        c(USSR, 1).star(),
        c(USSR, 3).star(),
        c(USSR, 3).star(),
        c(Neutral, 1).star(),
        c(US, 1).star(),
        c(Neutral, 2),
        c(US, 4).star(),
        c(US, 2).star(),
        c(US, 4).star(),
        c(Neutral, 2),
        c(US, 3).star(),
        c(US, 1).star(),
        c(US, 4).star(),
        c(USSR, 3).star(),
        c(US, 3),
        c(USSR, 2), // Decol
        c(Neutral, 4),
        c(Neutral, 1),
        c(USSR, 3).star(),
        c(Neutral, 4),
        c(US, 2).star(), // Formosan
    ];
    x
}

#[derive(Clone, Copy, PartialEq)]
pub enum Card {
    Pass = 0,
    Asia_Scoring = 1,
    Europe_Scoring,
    Middle_East_Scoring,
    Duck_and_Cover,
    Five_Year_Plan,
    The_China_Card,
    Socialist_Governments,
    Fidel,
    Vietnam_Revolts,
    Blockade,
    Korean_War,
    Romanian_Abdication,
    Arab_Israeli_War,
    Comecon,
    Nasser,
    Warsaw_Pact_Formed,
    De_Gaulle_Leads_France,
    Captured_Nazi_Scientist,
    Truman_Doctrine,
    Olympic_Games,
    NATO,
    Independent_Reds,
    Marshall_Plan,
    Indo_Pakistani_War,
    Containment,
    CIA_Created,
    US_Japan_Mutual_Defense_Pact,
    Suez_Crisis,
    East_European_Unrest,
    Decolonization,
    Red_Scare_Purge,
    UN_Intervention,
    De_Stalinization,
    Nuclear_Test_Ban,
    Formosan_Resolution = 35,
}

impl Card {
    /// Returns the list of event options an agent can select from this given
    /// card. If the return is None, the default behavior of just picking
    /// option 0 is sufficient.
    pub fn e_choices(&self, state: &GameState) -> Option<Vec<usize>> {
        use Card::*;
        match self {
            Blockade => {
                if !state.cards_above_value(Side::US, 3).is_empty() {
                    Some(vec![0, 1])
                } else {
                    None
                }
            }
            Olympic_Games => Some(vec![0, 1]),
            Independent_Reds => {
                let list = &[
                    CName::Yugoslavia,
                    CName::Romania,
                    CName::Bulgaria,
                    CName::Hungary,
                    CName::Czechoslovakia,
                ];
                let vec: Vec<_> = list
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        if state.countries[*c as usize].has_influence(Side::USSR) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                if vec.is_empty() {
                    None
                } else {
                    Some(vec)
                }
            }
            UN_Intervention => {
                let side = *state.side();
                let hand = state.deck.hand(side);
                let vec: Vec<_> = hand
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        if c.side() == side.opposite() {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                if vec.is_empty() {
                    None
                } else {
                    Some(vec)
                }
            }
            _ => None,
        }
    }
    pub fn event(&self, state: &mut GameState, choice: usize) -> bool {
        use Card::*;
        // let att = self.att();
        if !self.can_event(state) {
            return false;
        }
        match self {
            Asia_Scoring => {
                Region::Asia.score(state);
            }
            Europe_Scoring => {
                Region::Europe.score(state);
            }
            Middle_East_Scoring => {
                Region::MiddleEast.score(state);
            }
            Duck_and_Cover => {
                state.defcon -= 1;
                state.vp += 5 - state.defcon;
            }
            Five_Year_Plan => {
                let index = state.random_card(Side::USSR);
                let card = state.deck.ussr_hand()[index];
                if card.att().side == Side::US {
                    let x = Decision::new(Side::US, Action::Event(card, None), &[]);
                    state.pending_actions.push(x);
                    state.deck.play_card(Side::USSR, index, true);
                } else {
                    state.discard_card(Side::USSR, index);
                }
            }
            Socialist_Governments => {
                let x = Decision::new(
                    Side::USSR,
                    Action::Remove(Side::US, 1),
                    &country::WESTERN_EUROPE,
                );
                state.set_limit(2);
                state.pending_actions.push(x.clone());
                state.pending_actions.push(x.clone());
                state.pending_actions.push(x);
            }
            Fidel => {
                state.remove_all(Side::US, CName::Cuba);
                state.control(Side::USSR, CName::Cuba);
            }
            Vietnam_Revolts => state.ussr_effects.push(Effect::VietnamRevolts),
            Blockade => {
                if choice == 0 {
                    state.remove_all(Side::US, CName::WGermany);
                } else {
                    state.pending_actions.push(Decision::new(
                        Side::US,
                        Action::Discard(Side::US, 3),
                        &[],
                    ))
                }
            }
            Korean_War => {
                let index = CName::SKorea as usize;
                state.add_mil_ops(Side::USSR, 2);
                let roll = state.roll();
                if state.war_target(Side::USSR, index, roll) {
                    state.vp -= 2;
                }
            }
            Romanian_Abdication => {
                state.remove_all(Side::US, CName::Romania);
                state.control(Side::USSR, CName::Romania);
            }
            Arab_Israeli_War => {
                let index = CName::Israel as usize;
                let mut roll = state.roll();
                // This war is special, and includes the country itself
                if state.is_controlled(Side::US, index) {
                    roll -= 1;
                }
                state.add_mil_ops(Side::USSR, 2);
                if state.war_target(Side::USSR, index, roll) {
                    state.vp -= 2;
                }
            }
            Comecon => {
                let x = Decision::new(
                    Side::USSR,
                    Action::Place(Side::USSR, 1, false),
                    &country::EASTERN_EUROPE,
                );
                state.set_limit(1);
                state.pending_actions.push(x.clone());
                state.pending_actions.push(x.clone());
                state.pending_actions.push(x.clone());
                state.pending_actions.push(x);
            }
            Nasser => {
                let c = &mut state.countries[CName::Egypt as usize];
                c.ussr += 2;
                c.us /= 2;
            }
            Warsaw_Pact_Formed => {
                if !state.us_effects.contains(&Effect::AllowNato) {
                    state.us_effects.push(Effect::AllowNato);
                }
                if choice == 0 {
                    for _ in 0..4 {
                        state.pending_actions.push(Decision::new(
                            Side::USSR,
                            Action::RemoveAll(Side::US, true),
                            &country::EASTERN_EUROPE[..],
                        ));
                    }
                } else {
                    state.set_limit(2);
                    for _ in 0..5 {
                        state.pending_actions.push(Decision::new(
                            Side::USSR,
                            Action::Place(Side::USSR, 1, true),
                            &country::EASTERN_EUROPE[..],
                        ));
                    }
                }
            }
            De_Gaulle_Leads_France => {
                let c = &mut state.countries[CName::France as usize];
                let remove = std::cmp::min(2, c.us);
                c.us -= remove;
                c.ussr += 1;
                state.ussr_effects.push(Effect::DeGaulle);
            }
            Captured_Nazi_Scientist => {
                state.space_card(*state.side(), 1); // Todo ensure state.side is accurate
            }
            Truman_Doctrine => state.pending_actions.push(Decision::new(
                Side::US,
                Action::RemoveAll(Side::USSR, false),
                &country::EUROPE[..],
            )),
            Olympic_Games => {
                if choice == 0 {
                    let mut ussr_roll = 0;
                    let mut us_roll = 0;
                    while ussr_roll == us_roll {
                        ussr_roll = state.roll();
                        us_roll = state.roll();
                        if let Side::USSR = state.side() {
                            ussr_roll += 2;
                        } else {
                            us_roll += 2;
                        }
                    }
                    if us_roll > ussr_roll {
                        state.vp += 2;
                    } else {
                        state.vp -= 2;
                    }
                } else {
                    state.defcon -= 1;
                    let x = Decision::conduct_ops(*state.side(), 4);
                    state.pending_actions.push(x);
                }
            }
            NATO => {
                // let index = state
                //     .has_effect(Side::US, Effect::AllowNato)
                //     .expect("Already checked");
                // state.clear_effect(Side::US, index);
                state.us_effects.push(Effect::Nato);
            }
            Independent_Reds => {
                // Todo figure out if afterstates is too resource intensive
                let list = &[
                    CName::Yugoslavia,
                    CName::Romania,
                    CName::Bulgaria,
                    CName::Hungary,
                    CName::Czechoslovakia,
                ];
                let index = list[choice] as usize;
                let c = &mut state.countries[index];
                c.us = c.ussr;
            }
            Marshall_Plan => {
                if !state.us_effects.contains(&Effect::AllowNato) {
                    state.us_effects.push(Effect::AllowNato);
                }
                state.set_limit(1);
                for _ in 0..7 {
                    state.pending_actions.push(Decision::new(
                        Side::US,
                        Action::Place(Side::US, 1, false),
                        &country::WESTERN_EUROPE[..],
                    ));
                }
            }
            Indo_Pakistani_War => state.pending_actions.push(Decision::new(
                *state.side(),
                Action::War(*state.side(), false),
                &country::INDIA_PAKISTAN[..],
            )),
            Containment => state.us_effects.push(Effect::Containment),
            CIA_Created => {
                state.us_effects.push(Effect::USSR_Hand_Revealed);
                // Todo see what affects ops value
                state
                    .pending_actions
                    .push(Decision::conduct_ops(Side::US, 1));
            }
            US_Japan_Mutual_Defense_Pact => {
                state.control(Side::US, CName::Japan);
                // This effect is so useless I wonder if I should bother
                state.us_effects.push(Effect::US_Japan);
            }
            Suez_Crisis => {
                state.set_limit(2);
                for _ in 0..4 {
                    state.pending_actions.push(Decision::new(
                        Side::USSR,
                        Action::Remove(Side::US, 1),
                        &country::SUEZ,
                    ));
                }
            }
            East_European_Unrest => {
                let value = if state.turn <= 7 { 1 } else { 2 };
                state.set_limit(1);
                for _ in 0..3 {
                    state.pending_actions.push(Decision::new(
                        Side::US,
                        Action::Remove(Side::USSR, value),
                        &country::EASTERN_EUROPE,
                    ));
                }
            }
            Decolonization => {
                state.set_limit(1);
                for _ in 0..4 {
                    state.pending_actions.push(Decision::new(
                        Side::USSR,
                        Action::Place(Side::USSR, 1, true),
                        &country::DECOL,
                    ));
                }
            }
            Red_Scare_Purge => state.add_effect(*state.side(), Effect::RedScarePurge),
            UN_Intervention => {
                let card = state.deck.grab_card(*state.side(), choice);
                state.deck.discard_pile.push(card);
                let ops = card.ops();
                state
                    .pending_actions
                    .push(Decision::conduct_ops(*state.side(), ops));
            }
            De_Stalinization => {
                state.pending_actions.push(Decision::restriction_clear());
                // Todo fix &[] for place to be world
                for _ in 0..4 {
                    state.pending_actions.push(Decision::new(
                        Side::USSR,
                        Action::Place(Side::USSR, 1, false),
                        &[],
                    ))
                }
                // Limit only applies to placement. This may be a singular case
                state.pending_actions.push(Decision::limit_set(2));
                for _ in 0..4 {
                    state.pending_actions.push(Decision::new(
                        Side::USSR,
                        Action::Remove(Side::USSR, 1),
                        &[],
                    ));
                }
            }
            Nuclear_Test_Ban => {
                let vps = state.defcon - 2;
                match state.side() {
                    Side::US => state.vp += vps,
                    Side::USSR => state.vp -= vps,
                    _ => unimplemented!(),
                }
                state.defcon = std::cmp::min(5, state.defcon + 2);
            }
            Formosan_Resolution => state.us_effects.push(Effect::FormosanResolution),
            _ => {}
        }
        return true;
    }
    /// Returns whether a card can be evented, which is primarily relevant to
    /// whether or not a starred event will be removed if play by its opposing
    /// side.
    pub fn can_event(&self, state: &GameState) -> bool {
        use Card::*;
        match self {
            The_China_Card => false,
            Socialist_Governments => state.has_effect(Side::US, Effect::IronLady).is_none(),
            Arab_Israeli_War => state.has_effect(Side::US, Effect::CampDavid).is_none(),
            NATO => state.has_effect(Side::US, Effect::AllowNato).is_some(),
            _ => true, // todo make this accurate
        }
    }
    pub fn is_starred(&self) -> bool {
        self.att().starred
    }
    pub fn is_scoring(&self) -> bool {
        self.att().scoring
    }
    pub fn ops(&self) -> i8 {
        self.att().ops
    }
    pub fn side(&self) -> Side {
        self.att().side
    }
    /// Returns the attributes relevant to each unique card.
    fn att(&self) -> &'static Attributes {
        &ATT[*self as usize]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_cards() {
        let cards = super::init_cards();
        assert_eq!(cards.len(), 36);
    }
}
