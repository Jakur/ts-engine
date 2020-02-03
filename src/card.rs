#![allow(non_camel_case_types)]

use crate::action::{Action, Decision, Restriction};
use crate::country::{self, CName, Region, Side};
use crate::state::GameState;

use rand::seq::SliceRandom;
use rand::thread_rng;

lazy_static! {
    pub static ref ATT: Vec<Attributes> = init_cards();
}

#[derive(Clone)]
pub struct Deck {
    us_hand: Vec<Card>,
    ussr_hand: Vec<Card>,
    discard_pile: Vec<Card>,
    draw_pile: Vec<Card>,
    removed: Vec<Card>,
    china: Side,
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
        }
    }
    pub fn draw_cards(&mut self, target: usize) {
        let (us_goal, ussr_goal) = match self.china {
            Side::US => (target + 1, target),
            Side::USSR => (target, target + 1),
            Side::Neutral => unimplemented!(),
        };
        let mut pick_ussr = true;
        // Oscillating is relevant when reshuffles do occur
        while self.ussr_hand.len() < ussr_goal && self.us_hand.len() < us_goal {
            let next_card = self.draw_card();
            if pick_ussr {
                self.ussr_hand.push(next_card);
            } else {
                self.us_hand.push(next_card);
            }
            pick_ussr = !pick_ussr;
        }
        while self.ussr_hand.len() < ussr_goal {
            let c = self.draw_card();
            self.ussr_hand.push(c);
        }
        while self.us_hand.len() < us_goal {
            let c = self.draw_card();
            self.us_hand.push(c);
        }
    }
    fn draw_card(&mut self) -> Card {
        match self.draw_pile.pop() {
            Some(c) => c,
            None => {
                self.reshuffle();
                self.draw_card()
            }
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
    pub fn event(&self, state: &mut GameState) -> bool {
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
                    let x = Decision::new(Side::US, Action::Event(card, -1), &[]);
                    state.pending_actions.push(x);
                    state.deck.play_card(Side::USSR, index, true);
                } else {
                    state.discard_card(Side::USSR, index);
                }
            }
            Socialist_Governments => {
                let x = Decision::new(
                    Side::USSR,
                    Action::Remove(Side::US),
                    &country::WESTERN_EUROPE,
                );
                state.pending_actions.push(x.clone());
                state.pending_actions.push(x.clone());
                state.pending_actions.push(x);
                state.restrict = Some(Restriction::Limit(2));
            }
            Fidel => {
                state.remove_all(Side::US, CName::Cuba);
                state.control(Side::USSR, CName::Cuba);
            }
            Vietnam_Revolts => state.ussr_effects.push(Effect::VietnamRevolts),
            _ => {}
        }
        return true;
    }
    pub fn can_event(&self, state: &GameState) -> bool {
        use Card::*;
        match self {
            Socialist_Governments => state.has_effect(Side::US, Effect::IronLady).is_none(),
            _ => true, // todo make this accurate
        }
    }
    pub fn att(&self) -> &'static Attributes {
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
