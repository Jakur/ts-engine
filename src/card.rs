#![allow(non_camel_case_types)]

use crate::action::{Action, Allowed, Decision};
use crate::country::{self, CName, Country, Region, Side, Status};
use crate::state::{GameState, Period, TwilightRand};

use num_traits::FromPrimitive;

pub mod deck;
pub mod effect;
mod legal;
pub use deck::*;
pub use effect::*;

const NUM_CARDS: usize = Card::South_America_Scoring as usize + 1;

lazy_static! {
    static ref ATT: Vec<Attributes> = init_cards();
}

#[derive(Debug)]
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
        c(Neutral, 3),
        c(Neutral, 0).scoring(),
        c(Neutral, 0).star().scoring(),
        c(Neutral, 3),
        c(Neutral, 3).star(),
        c(US, 2).star(),
        c(USSR, 3).star(),
        c(Neutral, 3).star(),
        c(US, 3).star(),
        c(Neutral, 1), // Summit
        c(Neutral, 2).star(),
        c(Neutral, 2),
        c(US, 1).star(),
        c(Neutral, 2),
        c(USSR, 4).star(),
        c(USSR, 3).star(),
        c(USSR, 2).star(),
        c(USSR, 2),
        c(USSR, 1).star(),
        c(USSR, 2).star(), // Willy
        c(USSR, 4),
        c(Neutral, 4),
        c(USSR, 3).star(),
        c(USSR, 4).star(),
        c(USSR, 3).star(),
        c(USSR, 3),
        c(USSR, 1).star(),
        c(US, 2),
        c(US, 1).star(),
        c(US, 2).star(), // Camp David
        c(US, 2).star(),
        c(US, 2),
        c(US, 2).star(),
        c(Neutral, 2),
        c(US, 1).star(), // OAS
        c(US, 2).star(),
        c(US, 1).star(),
        c(US, 3),
        c(US, 2),
        c(USSR, 2), // Lib Theo
        c(US, 3).star(),
        c(US, 3).star(),
        c(US, 3).star(),
        c(Neutral, 0).scoring(),
        c(Neutral, 2),           // OSS
        c(Neutral, 0).scoring(), // End Mid War
    ];
    x
}

macro_rules! pa {
    ($s:ident, $d:ident) => {
        $s.add_pending($d);
    };
    ($s:ident, $d:expr) => {
        $s.add_pending($d);
    };
}

#[derive(Clone, Copy, PartialEq, FromPrimitive, Debug)]
pub enum Card {
    Dummy = 0,
    Asia_Scoring = 1,
    Europe_Scoring,
    Middle_East_Scoring,
    Duck_and_Cover,
    Five_Year_Plan, // 5, Ironically
    The_China_Card,
    Socialist_Governments,
    Fidel,
    Vietnam_Revolts,
    Blockade, // 10
    Korean_War,
    Romanian_Abdication,
    Arab_Israeli_War,
    Comecon,
    Nasser, // 15
    Warsaw_Pact_Formed,
    De_Gaulle_Leads_France,
    Captured_Nazi_Scientist,
    Truman_Doctrine,
    Olympic_Games, // 20
    NATO,
    Independent_Reds,
    Marshall_Plan,
    Indo_Pakistani_War,
    Containment, // 25
    CIA_Created,
    US_Japan_Mutual_Defense_Pact,
    Suez_Crisis,
    East_European_Unrest,
    Decolonization, // 30
    Red_Scare_Purge,
    UN_Intervention,
    De_Stalinization,
    Nuclear_Test_Ban,
    Formosan_Resolution, // 35
    Brush_War,
    Central_America_Scoring,
    Southeast_Asia_Scoring,
    Arms_Race,
    Cuban_Missile_Crisis, // 40
    Nuclear_Subs,
    Quagmire,
    SALT_Negotiations,
    Bear_Trap,
    Summit,                         // 45
    How_I_Learned_To_Stop_Worrying, // I'm sorry this is so long
    Junta,
    Kitchen_Debates,
    Missile_Envy,
    We_Will_Bury_You, // 50
    Brezhnev_Doctrine,
    Portuguese_Empire_Crumbles,
    South_African_Unrest,
    Allende,
    Willy_Brandt, // 55
    Muslim_Revolution,
    ABM_Treaty,
    Cultural_Revolution,
    Flower_Power,
    U2_Incident, // 60
    OPEC,
    Lone_Gunman,
    Colonial_Rear_Guards,
    Panama_Canal_Returned,
    Camp_David_Accords, // 65
    Puppet_Governments,
    Grain_Sales,
    John_Paul,
    Latin_American_Death_Squads,
    OAS_Founded, // 70
    Nixon_Plays_China,
    Sadat_Expels_Soviets,
    Shuttle_Diplomacy,
    The_Voice_Of_America,
    Liberation_Theology, // 75
    Ussuri_River_Skirmish,
    Ask_Not,
    Alliance_For_Progress,
    Africa_Scoring,
    One_Small_Step,        // 80
    South_America_Scoring, // End Mid War
}

impl Card {
    pub fn from_index(index: usize) -> Card {
        Self::from_usize(index).unwrap()
    }
    pub fn total() -> usize {
        NUM_CARDS
    }
    pub fn is_special(&self) -> bool {
        self.max_e_choices() > 1
    }
    pub fn influence_quantity(&self, state: &GameState, action: &Action, choice: usize) -> i8 {
        use Card::*;
        match self {
            Independent_Reds => state.countries[choice].ussr,
            East_European_Unrest => {
                if state.ar >= 8 {
                    2
                } else {
                    1
                }
            }
            Warsaw_Pact_Formed => {
                if let Action::Place = action {
                    1
                } else {
                    // Remove all
                    state.countries[choice].us
                }
            }
            Junta => 2,
            _ => 1,
        }
    }
    pub fn remove_quantity(&self, agent: Side, target: &Country, p: Period) -> (Side, i8) {
        use Card::*;
        let s = match self {
            De_Stalinization => Side::USSR,
            _ => agent.opposite(),
        };
        let q = match self {
            Warsaw_Pact_Formed | Truman_Doctrine => target.influence(s),
            East_European_Unrest => {
                if let Period::Late = p {
                    2
                } else {
                    1
                }
            }
            _ => 1,
        };
        let q = std::cmp::min(q, target.influence(s));
        (s, q)
    }
    pub fn max_e_choices(&self) -> usize {
        match self {
            Card::Blockade => 2,
            Card::Olympic_Games => 2,
            Card::Warsaw_Pact_Formed => 2,
            Card::Junta => 2,
            Card::South_African_Unrest => 2,
            _ => 1,
        }
    }
    /// Returns the list of event options an agent can select from this given
    /// card. If the return is None, the default behavior of just picking
    /// option 0 is sufficient.
    pub fn e_choices(&self, state: &GameState) -> Option<Vec<usize>> {
        use Card::*;
        match self {
            Blockade => {
                if state.cards_at_least(Side::US, 3).is_empty() {
                    Some(vec![0])
                } else {
                    Some(vec![0, 1])
                }
            }
            Olympic_Games => Some(vec![0, 1]),
            Warsaw_Pact_Formed => Some(vec![0, 1]),
            Junta => Some(vec![0, 1]),
            South_African_Unrest => Some(vec![0, 1]),
            _ => None,
        }
    }
    pub fn special_event<R: TwilightRand>(
        &self,
        state: &mut GameState,
        choice: usize,
        rng: &mut R,
    ) -> bool {
        use Card::*;
        let side = match self.side() {
            s @ Side::US | s @ Side::USSR => s,
            Side::Neutral => match state.current_event() {
                // Todo Star Wars
                Some(Card::Grain_Sales) => Side::US,
                _ => *state.side(),
            },
        };
        if !self.can_event(state, side) {
            return false;
        }
        match self {
            Blockade => {
                if choice == 0 {
                    state.remove_all(Side::US, CName::WGermany);
                } else {
                    let cards: Vec<_> = state
                        .cards_at_least(Side::US, 3)
                        .into_iter()
                        .map(|c| c as usize)
                        .collect();
                    let d = Decision::new(Side::US, Action::Discard, cards);
                    pa!(state, d);
                }
            }
            Warsaw_Pact_Formed => {
                if !state.has_effect(Side::US, Effect::AllowNato) {
                    state.add_effect(Side::US, Effect::AllowNato);
                }
                if choice == 0 {
                    let x = Decision::with_quantity(
                        Side::USSR,
                        Action::Remove,
                        not_opp_cont(&country::EASTERN_EUROPE[..], side, state),
                        4,
                    );
                    pa!(state, x);
                } else {
                    state.set_limit(2);
                    let x = Decision::with_quantity(
                        Side::USSR,
                        Action::Place,
                        &country::EASTERN_EUROPE[..],
                        5,
                    );
                    pa!(state, x);
                }
            }
            Olympic_Games => {
                if choice == 0 {
                    let mut ussr_roll = 0;
                    let mut us_roll = 0;
                    while ussr_roll == us_roll {
                        ussr_roll = rng.roll(Side::USSR);
                        us_roll = rng.roll(Side::US);
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
                    state.set_defcon(state.defcon() - 1);
                    let modifier = state.base_ops_offset(side);
                    let x = if modifier < 0 {
                        Decision::conduct_ops(side, 4 + modifier)
                    } else {
                        Decision::conduct_ops(side, 4)
                    };
                    pa!(state, x);
                }
            }
            Junta => {
                let action = if choice == 0 {
                    Action::Coup
                } else {
                    Action::Realignment
                };
                let legal = opp_has_inf(&country::LATIN_AMERICA, side, state);
                let ops = self.modified_ops(side, state);
                let d2 = Decision::with_quantity(side, action, legal, ops);
                pa!(state, d2);
                let d1 = Decision::new(side, Action::Place, &country::LATIN_AMERICA[..]);
                pa!(state, d1);
            }
            South_African_Unrest => {
                if choice == 0 {
                    state.countries[CName::SouthAfrica as usize].ussr += 2;
                } else {
                    let allowed = vec![CName::Angola as usize, CName::Botswana as usize];
                    let d = Decision::with_quantity(Side::USSR, Action::Place, allowed, 2);
                    pa!(state, d);
                }
            }
            _ => unimplemented!(),
        }
        true
    }
    pub fn event<R: TwilightRand>(&self, state: &mut GameState, rng: &mut R) -> bool {
        use Card::*;
        let side = match self.side() {
            s @ Side::US | s @ Side::USSR => s,
            Side::Neutral => match state.current_event() {
                // Todo Star Wars
                Some(Card::Grain_Sales) => Side::US,
                _ => *state.side(),
            },
        };
        if !self.can_event(state, side) {
            return false;
        }
        state.set_event(*self);
        if self.is_special() {
            let legal = self.e_choices(state).unwrap();
            let d = if *self == Card::Olympic_Games {
                // Opposite side decides which special outcome
                Decision::new(side.opposite(), Action::SpecialEvent, legal)
            } else {
                Decision::new(side, Action::SpecialEvent, legal)
            };
            let clear = Decision::new(Side::Neutral, Action::ClearEvent, &[]);
            pa!(state, clear);
            pa!(state, d);
            return true;
        }
        let clear = Decision::new(Side::Neutral, Action::ClearEvent, &[]);
        pa!(state, clear);
        match self {
            Dummy => panic!("Debug card evented!"),
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
                state.set_defcon(state.defcon() - 1);
                state.vp += 5 - state.defcon();
            }
            Five_Year_Plan => {
                let card = state.deck.random_card(Side::USSR, rng);
                if let Some(card) = card {
                    if card.att().side == Side::US {
                        let d = Decision::new_event(Side::USSR, card);
                        pa!(state, d);
                    }
                    state.discard_card(Side::USSR, card);
                }
            }
            Socialist_Governments => {
                let x = Decision::with_quantity(
                    Side::USSR,
                    Action::Remove,
                    opp_has_inf(&country::WESTERN_EUROPE[..], Side::USSR, &state),
                    3,
                );
                state.set_limit(2);
                pa!(state, x);
            }
            Fidel => {
                state.remove_all(Side::US, CName::Cuba);
                state.control(Side::USSR, CName::Cuba);
            }
            Vietnam_Revolts => state.add_effect(Side::USSR, Effect::VietnamRevolts),
            Korean_War => {
                let index = CName::SKorea as usize;
                state.add_mil_ops(Side::USSR, 2);
                let roll = rng.roll(Side::USSR);
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
                let mut roll = rng.roll(Side::USSR);
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
                    Action::Place,
                    not_opp_cont(&country::EASTERN_EUROPE[..], side, state),
                    4,
                );
                state.set_limit(1);
                pa!(state, x);
            }
            Nasser => {
                let c = &mut state.countries[CName::Egypt as usize];
                c.ussr += 2;
                c.us /= 2;
            }
            De_Gaulle_Leads_France => {
                let c = &mut state.countries[CName::France as usize];
                let remove = std::cmp::min(2, c.us);
                c.us -= remove;
                c.ussr += 1;
                state.add_effect(Side::USSR, Effect::DeGaulle);
            }
            Captured_Nazi_Scientist => {
                state.space_card(side, 1); // Todo ensure state.side is accurate
            }
            Truman_Doctrine => pa!(
                state,
                Decision::new(
                    Side::US,
                    Action::Remove,
                    not_opp_cont(&country::EUROPE[..], side, state),
                )
            ),
            NATO => state.add_effect(Side::US, Effect::Nato),
            Independent_Reds => {
                let allowed = opp_has_inf(&country::IND_REDS, Side::US, state);
                let x = Decision::new(Side::US, Action::Place, allowed);
                pa!(state, x);
            }
            Marshall_Plan => {
                if !state.has_effect(Side::US, Effect::AllowNato) {
                    state.add_effect(Side::US, Effect::AllowNato);
                }
                state.set_limit(1);
                let x = Decision::with_quantity(
                    Side::US,
                    Action::Place,
                    not_opp_cont(&country::WESTERN_EUROPE[..], side, state),
                    7,
                );
                pa!(state, x);
            }
            Indo_Pakistani_War => pa!(
                state,
                Decision::new(*state.side(), Action::War, &country::INDIA_PAKISTAN[..],)
            ),
            Containment => state.add_effect(Side::US, Effect::Containment),
            CIA_Created => {
                let offset = std::cmp::max(0, state.base_ops_offset(Side::US));
                state.add_effect(Side::US, Effect::USSR_Hand_Revealed);
                pa!(state, Decision::conduct_ops(Side::US, 1 + offset));
            }
            US_Japan_Mutual_Defense_Pact => {
                state.control(Side::US, CName::Japan);
                // This effect is so useless I wonder if I should bother
                state.add_effect(Side::US, Effect::US_Japan);
            }
            Suez_Crisis => {
                let x = Decision::with_quantity(
                    Side::USSR,
                    Action::Remove,
                    opp_has_inf(&country::SUEZ[..], side, state),
                    4,
                );
                state.set_limit(2);
                pa!(state, x);
            }
            East_European_Unrest => {
                state.set_limit(1);
                let x = Decision::with_quantity(
                    Side::US,
                    Action::Remove,
                    opp_has_inf(&country::EASTERN_EUROPE[..], side, state),
                    3,
                );
                pa!(state, x);
            }
            Decolonization => {
                state.set_limit(1);
                let x = Decision::with_quantity(Side::USSR, Action::Place, &country::DECOL[..], 4);
                pa!(state, x);
            }
            Red_Scare_Purge => state.add_effect(*state.side(), Effect::RedScarePurge),
            UN_Intervention => {
                let hand = state.deck.hand(side);
                let opp = side.opposite();
                let vec: Vec<_> = hand
                    .iter()
                    .copied()
                    .filter_map(|c| {
                        if c.side() == opp {
                            Some(c as usize)
                        } else {
                            None
                        }
                    })
                    .collect();
                if state.has_effect(Side::USSR, Effect::U2) {
                    state.vp -= 1;
                }
                let d = Decision::new(side, Action::Ops, vec);
                pa!(state, d);
            }
            De_Stalinization => {
                state.set_limit(2);
                let dest: Vec<_> = state
                    .valid_countries()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        if c.controller() != Side::US {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                let x = Decision::with_quantity(Side::USSR, Action::Place, dest, 4);
                let source: Vec<_> = state
                    .valid_countries()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        if c.has_influence(Side::USSR) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                let y = Decision::with_quantity(Side::USSR, Action::Remove, source, 4);
                pa!(state, x);
                pa!(state, y);
            }
            Nuclear_Test_Ban => {
                let vps = state.defcon() - 2;
                match state.side() {
                    Side::US => state.vp += vps,
                    Side::USSR => state.vp -= vps,
                    _ => unimplemented!(),
                }
                state.set_defcon(state.defcon() + 2);
            }
            Formosan_Resolution => state.add_effect(Side::US, Effect::FormosanResolution),
            Brush_War => pa!(
                state,
                Decision::new(side, Action::War, state.legal_war(side))
            ),
            Central_America_Scoring => {
                Region::CentralAmerica.score(state);
            }
            Southeast_Asia_Scoring => {
                Region::SoutheastAsia.score(state);
            }
            Arms_Race => {
                if state.mil_ops(side) > state.mil_ops(side.opposite()) {
                    if state.mil_ops(side) >= state.defcon() {
                        state.vp += 3;
                    } else {
                        state.vp += 1
                    }
                }
            }
            Cuban_Missile_Crisis => {
                state.set_defcon(2);
                state.add_effect(side.opposite(), Effect::CubanMissileCrisis);
            }
            Nuclear_Subs => state.add_effect(side, Effect::NuclearSubs),
            Quagmire => state.add_effect(side.opposite(), Effect::Quagmire),
            SALT_Negotiations => {
                state.add_effect(side, Effect::SALT);
                state.add_effect(side.opposite(), Effect::SALT);
                let allowed: Vec<_> = state
                    .deck
                    .discard_pile()
                    .iter()
                    .map(|c| *c as usize)
                    .collect();
                let d = Decision::new(side, Action::RecoverCard, allowed);
                pa!(state, d)
            }
            Bear_Trap => state.add_effect(side.opposite(), Effect::BearTrap),
            Summit => {
                let mut ussr_roll = rng.roll(Side::USSR);
                let mut us_roll = rng.roll(Side::US);
                for r in Region::major_regions() {
                    let (status, _) = r.status(state, false);
                    match status[Side::US as usize] {
                        Status::Domination | Status::Control => us_roll += 1,
                        _ => {}
                    }
                    match status[Side::USSR as usize] {
                        Status::Domination | Status::Control => ussr_roll += 1,
                        _ => {}
                    }
                }
                let defcon: Vec<_> = [state.defcon() - 1, state.defcon(), state.defcon() + 1]
                    .iter()
                    .copied()
                    .filter_map(|x| {
                        if 1 <= x && x <= 5 {
                            Some(x as usize)
                        } else {
                            None
                        }
                    })
                    .collect();
                if us_roll > ussr_roll {
                    state.vp += 2;
                    pa!(state, Decision::new(Side::US, Action::ChangeDefcon, defcon));
                } else if ussr_roll > us_roll {
                    state.vp -= 2;
                    pa!(
                        state,
                        Decision::new(Side::USSR, Action::ChangeDefcon, defcon)
                    );
                }
            }
            How_I_Learned_To_Stop_Worrying => {
                let d = Decision::new(side, Action::ChangeDefcon, vec![1, 2, 3, 4, 5]);
                state.add_mil_ops(side, 5);
                pa!(state, d);
            }
            Missile_Envy => {
                let allowed: Vec<_> = state
                    .deck
                    .highest_ops(side.opposite())
                    .into_iter()
                    .map(|c| c as usize)
                    .collect();
                let d = Decision::new(side.opposite(), Action::ChooseCard, allowed);
                pa!(state, d);
                // Set missile envy effect when the above decision resolves
            }
            We_Will_Bury_You => {
                state.set_defcon(state.defcon() - 1);
                state.add_effect(Side::US, Effect::WWBY);
            }
            Kitchen_Debates => state.vp += 2,
            Brezhnev_Doctrine => state.add_effect(Side::US, Effect::Brezhnev),
            Portuguese_Empire_Crumbles => {
                state.countries[CName::Angola as usize].ussr += 2;
                state.countries[CName::SEAfricanStates as usize].ussr += 2;
            }
            Allende => state.countries[CName::Chile as usize].ussr += 2,
            Willy_Brandt => {
                state.vp -= 1;
                state.countries[CName::WGermany as usize].ussr += 1;
                state.add_effect(Side::USSR, Effect::WillyBrandt);
            }
            Muslim_Revolution => {
                let allowed = Allowed::new_lazy(legal::muslim_rev);
                let d = Decision::new(Side::USSR, Action::Remove, allowed);
                pa!(state, d.clone());
                pa!(state, d);
            }
            ABM_Treaty => {
                state.set_defcon(state.defcon() + 1);
                let ops = self.modified_ops(side, state);
                pa!(state, Decision::conduct_ops(side, ops));
            }
            Cultural_Revolution => {
                if let Side::US = state.deck.china() {
                    state.deck.play_china();
                    state.deck.turn_china_up();
                } else {
                    state.vp -= 1;
                }
            }
            Flower_Power => state.add_effect(Side::USSR, Effect::FlowerPower),
            U2_Incident => {
                state.vp -= 1;
                state.add_effect(Side::USSR, Effect::U2);
            }
            OPEC => {
                let count = country::OPEC.iter().fold(0, |acc, c| {
                    if let Side::USSR = state.countries[*c].controller() {
                        acc + 1
                    } else {
                        acc
                    }
                });
                state.vp -= count;
            }
            Lone_Gunman => {
                let ops = self.modified_ops(Side::USSR, state);
                state.add_effect(Side::USSR, Effect::US_Hand_Revealed);
                pa!(state, Decision::conduct_ops(Side::USSR, ops));
            }
            Colonial_Rear_Guards => {
                // USA Decol
                state.set_limit(1);
                let x = Decision::with_quantity(Side::US, Action::Place, &country::DECOL[..], 4);
                pa!(state, x);
            }
            Panama_Canal_Returned => {
                state.countries[CName::Panama as usize].us += 1;
                state.countries[CName::CostaRica as usize].us += 1;
                state.countries[CName::Venezuela as usize].us += 1;
            }
            Camp_David_Accords => {
                state.vp += 1;
                state.countries[CName::Israel as usize].us += 1;
                state.countries[CName::Jordan as usize].us += 1;
                state.countries[CName::Egypt as usize].us += 1;
                state.add_effect(Side::US, Effect::CampDavid);
            }
            Puppet_Governments => {
                state.set_limit(1);
                let legal: Vec<_> = state
                    .valid_countries()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        if c.us == 0 && c.ussr == 0 {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                let d = Decision::with_quantity(Side::US, Action::Place, legal, 3);
                pa!(state, d);
            }
            Grain_Sales => {
                let hit = rng.card_from_hand(&state.deck, Side::USSR);
                if let Some(card) = hit {
                    let d = Decision::new(
                        Side::US,
                        Action::ChooseCard,
                        vec![Card::Dummy as usize, card as usize],
                    );
                    pa!(state, d);
                } else {
                    let ops = 2 + state.base_ops_offset(Side::US);
                    let d = Decision::conduct_ops(Side::US, ops);
                    pa!(state, d);
                }
            }
            John_Paul => {
                let poland = &mut state.countries[CName::Poland as usize];
                poland.ussr = std::cmp::max(poland.ussr - 2, 0);
                poland.us += 1;
                state.add_effect(Side::US, Effect::AllowSolidarity);
            }
            Latin_American_Death_Squads => {
                state.add_effect(side, Effect::LatinAmericanPlus);
                state.add_effect(side.opposite(), Effect::LatinAmericanMinus);
            }
            OAS_Founded => {
                let d = Decision::with_quantity(
                    Side::US,
                    Action::Place,
                    &country::LATIN_AMERICA[..],
                    2,
                );
                pa!(state, d);
            }
            Nixon_Plays_China => {
                if let Side::USSR = state.deck.china() {
                    state.deck.play_china();
                } else {
                    state.vp += 2;
                }
            }
            Sadat_Expels_Soviets => {
                let egypt = &mut state.countries[CName::Egypt as usize];
                egypt.ussr = 0;
                egypt.us += 1;
            }
            Shuttle_Diplomacy => state.add_effect(Side::US, Effect::ShuttleDiplomacy),
            The_Voice_Of_America => {
                state.set_limit(2);
                let start = country::EUROPE.last().unwrap() + 1;
                let legal: Vec<_> = (start..country::NUM_COUNTRIES - 2)
                    .filter(|x| state.countries[*x].has_influence(Side::USSR))
                    .collect();
                let d = Decision::with_quantity(Side::US, Action::Remove, legal, 4);
                pa!(state, d);
            }
            Liberation_Theology => {
                state.set_limit(2);
                let legal = &country::CENTRAL_AMERICA[..];
                let d = Decision::with_quantity(Side::USSR, Action::Place, legal, 3);
                pa!(state, d);
            }
            Ussuri_River_Skirmish => {
                if let Side::US = state.deck.china() {
                    state.set_limit(2);
                    let legal = &country::ASIA[..];
                    let d = Decision::with_quantity(Side::USSR, Action::Place, legal, 4);
                    pa!(state, d);
                } else {
                    state.deck.play_china();
                    state.deck.turn_china_up();
                }
            }
            Ask_Not => {
                let allowed: Vec<_> = state.deck.us_hand().iter().map(|x| *x as usize).collect();
                let q = allowed.len() as i8;
                let d = Decision::with_quantity(Side::US, Action::ChooseCard, allowed, q);
                pa!(state, d);
            }
            Alliance_For_Progress => {
                let count = &country::LATIN_AMERICA
                    .iter()
                    .filter(|x| {
                        let c = &state.countries[**x];
                        c.bg && c.controller() == Side::US
                    })
                    .count();
                state.vp += *count as i8;
            }
            Africa_Scoring => {
                Region::Africa.score(state);
            }
            One_Small_Step => {
                let index = side as usize;
                if state.space[index] < 7 {
                    // If you're at space 7, your final location is only +1
                    state.space[index] += 1;
                }
                state.space_card(side, 1); // 1 is a perfect roll
            }
            South_America_Scoring => {
                Region::SouthAmerica.score(state);
            }
            The_China_Card => {}
            Olympic_Games | Blockade | Warsaw_Pact_Formed | Junta | South_African_Unrest => {
                unimplemented!()
            }
        }
        return true;
    }
    /// Returns whether a card can be evented, which is primarily relevant to
    /// whether or not a starred event will be removed if play by its opposing
    /// side.
    pub fn can_event(&self, state: &GameState, eventer: Side) -> bool {
        use Card::*;
        match self {
            The_China_Card => false,
            Socialist_Governments => !state.has_effect(Side::US, Effect::IronLady),
            Arab_Israeli_War => !state.has_effect(Side::US, Effect::CampDavid),
            NATO => state.has_effect(Side::US, Effect::AllowNato),
            UN_Intervention => {
                let opp = state.side().opposite();
                // Eventable if the side has any card with an opponent's event
                state
                    .deck
                    .hand(*state.side())
                    .iter()
                    .any(|c| c.side() == opp)
            }
            Kitchen_Debates => {
                // Todo figure out if it's worth caching the bg list
                let us_lead = state
                    .valid_countries()
                    .iter()
                    .filter(|c| c.bg)
                    .fold(0, |acc, c| match c.controller() {
                        Side::US => acc + 1,
                        Side::USSR => acc - 1,
                        _ => acc,
                    });
                us_lead > 0
            }
            Willy_Brandt => !state.has_effect(Side::US, Effect::TearDown),
            Muslim_Revolution => !state.has_effect(Side::US, Effect::AWACS),
            OPEC => !state.has_effect(Side::US, Effect::NoOpec),
            One_Small_Step => {
                state.space[eventer as usize] < state.space[eventer.opposite() as usize]
            }
            _ => true, // todo make this accurate
        }
    }
    pub fn can_headline(&self) -> bool {
        match self {
            Card::The_China_Card | Card::UN_Intervention => false,
            _ => true,
        }
    }
    pub fn is_starred(&self) -> bool {
        self.att().starred
    }
    pub fn is_scoring(&self) -> bool {
        self.att().scoring
    }
    pub fn is_war(&self) -> bool {
        match self {
            Card::Arab_Israeli_War
            | Card::Korean_War
            | Card::Indo_Pakistani_War
            | Card::Brush_War => true,
            _ => false,
        }
    }
    pub fn modified_ops(&self, side: Side, state: &GameState) -> i8 {
        let offset = state.base_ops_offset(side);
        self.ops(offset)
    }
    pub fn ops(&self, offset: i8) -> i8 {
        let x = self.base_ops() + offset;
        if offset > 0 {
            std::cmp::min(4, x)
        } else if offset < 0 {
            std::cmp::max(1, x)
        } else {
            x
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

fn not_opp_cont(slice: &[usize], side: Side, state: &GameState) -> Vec<usize> {
    slice
        .iter()
        .copied()
        .filter(|&x| !state.is_controlled(side.opposite(), x))
        .collect()
}

fn opp_has_inf(slice: &[usize], side: Side, state: &GameState) -> Vec<usize> {
    let opp = side.opposite();
    slice
        .iter()
        .copied()
        .filter(|&x| state.countries[x].has_influence(opp))
        .collect()
}

impl From<Card> for usize {
    fn from(c: Card) -> Self {
        c as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;
    #[test]
    fn check_cards() {
        // Cards are sequential and do not overflow u8
        let cards: Vec<_> = (0..)
            .map(|x| Card::from_u8(x))
            .take_while(|x| x.is_some())
            .collect();
        assert_eq!(cards.len(), NUM_CARDS); // Make sure num cards is actually right
        let atts = init_cards();
        assert_eq!(atts.len(), NUM_CARDS);
    }
    #[test]
    fn check_special_cards() {
        let state = GameState::new();
        for c in (1..Card::total()).map(|i| Card::from_index(i)) {
            let e_choices = c.e_choices(&state);
            if c.is_special() {
                assert!(e_choices.is_some()); // Some, even in dummy state
            } else {
                assert!(e_choices.is_none());
            }
        }
    }
}
