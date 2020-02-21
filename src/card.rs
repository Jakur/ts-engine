#![allow(non_camel_case_types)]

use crate::action::{Action, Decision};
use crate::country::{self, CName, Region, Side};
use crate::state::GameState;

use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod deck;
pub mod effect;
pub use deck::*;
pub use effect::*;

lazy_static! {
    static ref ATT: Vec<Attributes> = init_cards();
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

#[derive(Clone, Copy, PartialEq, FromPrimitive)]
pub enum Card {
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
    pub fn total() -> usize {
        todo!() // Until we add in the last card
    }
    /// Returns the list of event options an agent can select from this given
    /// card. If the return is None, the default behavior of just picking
    /// option 0 is sufficient.
    pub fn e_choices(&self, state: &GameState) -> Option<Vec<usize>> {
        use Card::*;
        match self {
            Blockade => {
                let mut discards: Vec<_> = state.cards_at_least(Side::US, 3).into_iter().map(|i| {
                    let card_id = state.deck.us_hand()[i] as usize;
                    card_id
                }).collect();
                discards.push(0); // Do not discard
                Some(discards)
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
                    .filter_map(|c| {
                        let i = *c as usize;
                        if state.countries[i].has_influence(Side::USSR) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                Some(vec)
            }
            UN_Intervention => {
                let side = *state.side();
                let hand = state.deck.hand(side);
                let vec: Vec<_> = hand
                    .iter()
                    .filter_map(|c| {
                        if c.side() == side.opposite() {
                            Some(*c as usize)
                        } else {
                            None
                        }
                    })
                    .collect();
                Some(vec)
            }
            _ => None,
        }
    }
    pub fn event(
        &self,
        state: &mut GameState,
        choice: usize,
        pending_actions: &mut Vec<Decision>,
    ) -> bool {
        use Card::*;
        use num_traits::FromPrimitive;
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
                let card = state.deck.random_card(Side::USSR);
                if let Some(&card) = card {
                    if card.att().side == Side::US {
                        // Todo find out of the US really has agency in these decisions?
                        // Just Chernobyl?
                        let x = Decision::new(Side::US, Action::Event(card, None), &[]);
                        pending_actions.push(x);
                    }
                    state.discard_card(Side::USSR, card);
                }
            }
            Socialist_Governments => {
                let x = Decision::with_quantity(
                    Side::USSR,
                    Action::Remove(Side::US, 1),
                    &country::WESTERN_EUROPE[..],
                    3
                );
                state.set_limit(2, pending_actions);
                pending_actions.push(x);
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
                    let card = Card::from_usize(choice).expect("Already checked");
                    state.deck.play_card(Side::US, card);
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
                let x = Decision::with_quantity(
                    Side::USSR,
                    Action::Place(Side::USSR, 1, false),
                    &country::EASTERN_EUROPE[..],
                    4
                );
                state.set_limit(1, pending_actions);
                pending_actions.push(x);
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
                    let x = Decision::with_quantity(
                        Side::USSR,
                        Action::RemoveAll(Side::US, true),
                        &country::EASTERN_EUROPE[..],
                        4
                    );
                    pending_actions.push(x);
                } else {
                    state.set_limit(2, pending_actions);
                    let x = Decision::with_quantity(
                        Side::USSR,
                        Action::Place(Side::USSR, 1, true),
                        &country::EASTERN_EUROPE[..],
                        4
                    );
                    pending_actions.push(x);
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
            Truman_Doctrine => pending_actions.push(Decision::new(
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
                    pending_actions.push(x);
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
                let c = &mut state.countries[choice];
                c.us = c.ussr;
            }
            Marshall_Plan => {
                if !state.us_effects.contains(&Effect::AllowNato) {
                    state.us_effects.push(Effect::AllowNato);
                }
                state.set_limit(1, pending_actions);
                let x = Decision::with_quantity(
                    Side::US,
                    Action::Place(Side::US, 1, false),
                    &country::WESTERN_EUROPE[..],
                    7
                );
                pending_actions.push(x);
            }
            Indo_Pakistani_War => pending_actions.push(Decision::new(
                *state.side(),
                Action::War(*state.side(), false),
                &country::INDIA_PAKISTAN[..],
            )),
            Containment => state.us_effects.push(Effect::Containment),
            CIA_Created => {
                let offset = std::cmp::max(0, state.base_ops_offset(Side::US));
                state.us_effects.push(Effect::USSR_Hand_Revealed);
                pending_actions.push(Decision::conduct_ops(Side::US, 1 + offset));
            }
            US_Japan_Mutual_Defense_Pact => {
                state.control(Side::US, CName::Japan);
                // This effect is so useless I wonder if I should bother
                state.us_effects.push(Effect::US_Japan);
            }
            Suez_Crisis => {
                let x = Decision::with_quantity(
                    Side::USSR,
                    Action::Remove(Side::US, 1),
                    &country::SUEZ[..],
                    4
                );
                state.set_limit(2, pending_actions);
                pending_actions.push(x);
            }
            East_European_Unrest => {
                let value = if state.turn <= 7 { 1 } else { 2 };
                state.set_limit(1, pending_actions);
                let x = Decision::with_quantity(
                    Side::US,
                    Action::Remove(Side::USSR, value),
                    &country::EASTERN_EUROPE[..],
                    3
                );
                pending_actions.push(x);
            }
            Decolonization => {
                state.set_limit(1, pending_actions);
                let x = Decision::with_quantity(
                    Side::USSR,
                    Action::Place(Side::USSR, 1, true),
                    &country::DECOL[..],
                    4
                );
                pending_actions.push(x);
            }
            Red_Scare_Purge => state.add_effect(*state.side(), Effect::RedScarePurge),
            UN_Intervention => {
                let card = state.deck.hand(*state.side())[choice];
                let ops = card.modified_ops(*state.side(), state);
                state.deck.play_card(*state.side(), card);
                pending_actions.push(Decision::conduct_ops(*state.side(), ops));
            }
            De_Stalinization => {
                state.set_limit(2, pending_actions);
                let allowed: Vec<_> = state.valid_countries().iter().enumerate().filter_map(|(i, c)| {
                    if c.controller() != Side::US {
                        Some(i)
                    } else {
                        None
                    }
                }).collect();
                let x = Decision::with_quantity(Side::USSR, Action::Destal, allowed, 4);
                pending_actions.push(x);
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
    pub fn can_headline(&self, state: &GameState) -> bool {
        // Todo make sure this is right
        self.can_event(state) && *self != Card::UN_Intervention
    }
    pub fn is_starred(&self) -> bool {
        self.att().starred
    }
    pub fn is_scoring(&self) -> bool {
        self.att().scoring
    }
    pub fn modified_ops(&self, side: Side, state: &GameState) -> i8 {
        let base_ops = self.base_ops();
        let offset = state.base_ops_offset(side);
        if offset == 0 {
            base_ops
        } else if offset == -1 {
            std::cmp::max(1, base_ops + offset)
        } else { // +1
            std::cmp::min(4, base_ops + offset)
        }
    }
    pub fn base_ops(&self) -> i8 {
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
    use super::*;
    use num_traits::FromPrimitive;
    #[test]
    fn check_cards() {
        let atts = init_cards();
        assert_eq!(atts.len(), 36);
        let cards: Vec<_> = (1..36).map(|x| Card::from_u8(x)).collect();
        for c in cards {
            assert!(c.is_some());
        }
    }
}
