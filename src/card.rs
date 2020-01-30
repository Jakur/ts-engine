#![allow(non_camel_case_types)]

use crate::action::{Action, Decision, Restriction};
use crate::country::{self, CName, Region, Side};
use crate::state::GameState;

lazy_static! {
    pub static ref ATT: Vec<Attributes> = init_cards();
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
                let card = state.ussr_hand[index];
                if card.att().side == Side::US {
                    let x = Decision::new(Side::US, Action::Event(card), &[]);
                    state.pending_actions.push(x);
                }
                state.discard_card(Side::USSR, index);
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
