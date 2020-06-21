#![allow(non_camel_case_types)]

use crate::action::{Action, Allowed, Decision};
use crate::country::{self, CName, Country, Region, Side, Status};
use crate::state::{GameState, Period, TwilightRand};

use num_traits::FromPrimitive;

pub mod deck;
pub mod effect;
pub mod legal;
pub use deck::*;
pub use effect::*;

const NUM_CARDS: usize = Card::AWACS as usize + 1;

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
        c(USSR, 3).star(),
        c(US, 3).star(),
        c(US, 2).star(),
        c(US, 2).star(), // Star Wars
        c(US, 3).star(),
        c(USSR, 3).star(),
        c(USSR, 2).star(),
        c(US, 4).star(),
        c(USSR, 4).star(),
        c(USSR, 2).star(),
        c(Neutral, 2),
        c(USSR, 2).star(),
        c(US, 3).star(),
        c(USSR, 2), // LADS
        c(US, 3).star(),
        c(US, 3).star(),
        c(USSR, 3).star(),
        c(USSR, 3).star(),
        c(Neutral, 4).star(),
        c(US, 2).star(),
        c(Neutral, 2).star(), // End Late War
        c(US, 2),             // Defectors
        c(USSR, 2),
        c(US, 2), // Special Relationship
        c(US, 3).star(),
        c(USSR, 3),
        c(US, 2).star(),
        c(USSR, 2).star(),
        c(US, 3).star(), // AWACS
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
    Iranian_Hostage_Crisis,
    The_Iron_Lady,
    Reagan_Bombs_Libya,
    Star_Wars, // 85
    North_Sea_Oil,
    The_Reformer,
    Marine_Barracks_Bombing,
    Soviets_Shoot_Down_KAL,
    Glasnost, // 90
    Ortega_Elected,
    Terrorism,
    Iran_Contra_Scandal,
    Chernobyl,
    Latin_American_Debt_Crisis, // 95
    Tear_Down_This_Wall,
    An_Evil_Empire,
    Aldrich_Ames_Remix,
    Pershing_II_Deployed,
    Wargames, // 100
    Solidarity,
    Iran_Iraq_War,        // End Late War
    Defectors,            // 103
    The_Cambridge_Five,   // Begin Optionals
    Special_Relationship, // 105
    NORAD,
    Che,
    Our_Man_In_Tehran,
    Yuri_And_Samantha,
    AWACS,
}

impl Card {
    pub fn from_index(index: usize) -> Card {
        Self::from_usize(index).unwrap()
    }
    pub const fn total() -> usize {
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
            Card::Latin_American_Debt_Crisis => 2,
            Card::Wargames => 2,
            _ => 1,
        }
    }
    /// Returns the list of event options an agent can select from this given
    /// card. If the return is None, the default behavior of just picking
    /// option 0 is sufficient.
    pub fn e_choices(&self, state: &GameState) -> Option<Vec<usize>> {
        use Card::*;
        match self {
            Blockade | Latin_American_Debt_Crisis => {
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
            Wargames => Some(vec![0, 1]),
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
            Latin_American_Debt_Crisis => {
                if choice == 0 {
                    state.set_limit(1);
                    let (sa_start, _) = Region::SouthAmerica.low_high();
                    let allowed: Vec<_> = country::SOUTH_AMERICA
                        .iter()
                        .filter_map(|&x| {
                            if state.countries[x].has_influence(Side::USSR) {
                                Some(x - sa_start)
                            } else {
                                None
                            }
                        })
                        .collect();
                    let d = Decision::with_quantity(Side::USSR, Action::DoubleInf, allowed, 2);
                    pa!(state, d);
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
            Wargames => {
                // The only winning move is not to play
                if choice != 0 {
                    if let Side::USSR = side {
                        state.vp += 6;
                    } else {
                        state.vp -= 6;
                    }
                    // End Game
                    if state.vp < 0 {
                        state.vp = -20;
                    } else {
                        state.vp = 20;
                    }
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
            Vietnam_Revolts => {
                state.add_effect(Side::USSR, Effect::VietnamRevolts);
                state.countries[CName::Vietnam as usize].ussr += 2
            }
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
            Quagmire => {
                state.add_effect(side.opposite(), Effect::Quagmire);
                if let Some(i) = state.effect_pos(Side::US, Effect::Norad) {
                    state.clear_effect(Side::US, i);
                }
            }
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
                for r in Region::major_regions().iter() {
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
            Iranian_Hostage_Crisis => {
                let iran = &mut state.countries[CName::Iran as usize];
                iran.us = 0;
                iran.ussr += 2;
                state.add_effect(Side::USSR, Effect::TerrorismPlus);
            }
            The_Iron_Lady => {
                state.countries[CName::UK as usize].ussr = 0;
                let arg = &mut state.countries[CName::Argentina as usize];
                if arg.ussr == 0 {
                    state.iron_lady = true; // Flag for the access weirdness
                }
                arg.ussr += 1;
                state.vp += 1;
                state.add_effect(Side::US, Effect::IronLady);
            }
            Reagan_Bombs_Libya => {
                state.vp += state.countries[CName::Libya as usize].ussr / 2;
            }
            Star_Wars => {
                let mut allowed: Vec<_> = state
                    .deck
                    .discard_pile()
                    .iter()
                    .filter_map(|c| {
                        if state.ar != 0 || c.can_headline() {
                            Some(*c as usize)
                        } else {
                            None
                        }
                    })
                    .collect();
                // Todo figure out if pending discard is an unnecessary abstraction
                if let Some(c) = state.deck.pending_discard().iter().find(|c| *c != self) {
                    allowed.push(*c as usize);
                }
                let d = Decision::new(Side::US, Action::Event, allowed);
                pa!(state, d);
            }
            North_Sea_Oil => {
                state.add_effect(Side::US, Effect::NorthSeaOil);
                state.add_effect(Side::USSR, Effect::NoOpec);
            }
            The_Reformer => {
                state.add_effect(Side::USSR, Effect::Reformer);
                state.set_limit(2);
                let allowed = &country::EUROPE[..];
                let q = if state.vp < 0 { 6 } else { 4 };
                let d = Decision::with_quantity(Side::USSR, Action::Place, allowed, q);
                pa!(state, d);
            }
            Marine_Barracks_Bombing => {
                state.countries[CName::Lebanon as usize].us = 0;
                let allowed = opp_has_inf(&country::MIDDLE_EAST, Side::USSR, state);
                let d = Decision::with_quantity(Side::USSR, Action::Remove, allowed, 2);
                pa!(state, d);
            }
            Soviets_Shoot_Down_KAL => {
                state.set_defcon(state.defcon() - 1);
                state.vp += 2;
                if let Side::US = state.countries[CName::SKorea as usize].controller() {
                    let ops = self.modified_ops(Side::US, state);
                    let d = Decision::conduct_ops(Side::US, ops);
                    pa!(state, d);
                }
            }
            Glasnost => {
                state.set_defcon(state.defcon() + 1);
                state.vp -= 2;
                if state.has_effect(Side::USSR, Effect::Reformer) {
                    let ops = self.modified_ops(Side::USSR, state);
                    let d = Decision::conduct_ops(Side::USSR, ops);
                    pa!(state, d);
                }
            }
            Ortega_Elected => {
                let nic = CName::Nicaragua as usize;
                state.countries[nic].us = 0;
                let allowed = opp_has_inf(&country::EDGES[nic], Side::USSR, state);
                let ops = self.modified_ops(Side::USSR, state);
                let d = Decision::with_quantity(Side::USSR, Action::Coup, allowed, ops);
                pa!(state, d);
            }
            Terrorism => {
                let opp = side.opposite();
                let card = state.deck.random_card(opp, rng);
                if let Some(c) = card {
                    state.discard_card(opp, c);
                }
                if side == Side::USSR && state.has_effect(Side::USSR, Effect::TerrorismPlus) {
                    let card2 = state.deck.random_card(opp, rng);
                    if let Some(c) = card2 {
                        state.discard_card(opp, c);
                    }
                }
            }
            Iran_Contra_Scandal => state.add_effect(Side::USSR, Effect::IranContra),
            Chernobyl => {
                let allowed: Vec<_> = (0..6).collect();
                let d = Decision::new(Side::US, Action::BlockRegion, allowed);
                pa!(state, d);
            }
            Tear_Down_This_Wall => {
                state.countries[CName::EGermany as usize].us += 3;
                let ops = self.modified_ops(Side::US, state);
                let d = Decision::conduct_ops(Side::US, ops);
                if let Some(i) = state.effect_pos(Side::USSR, Effect::WillyBrandt) {
                    state.clear_effect(Side::USSR, i);
                }
                state.add_effect(Side::US, Effect::TearDown);
                pa!(state, d);
            }
            An_Evil_Empire => {
                state.vp += 1;
                if let Some(i) = state.effect_pos(Side::USSR, Effect::FlowerPower) {
                    state.clear_effect(Side::USSR, i);
                }
                state.add_effect(Side::US, Effect::EvilEmpire);
            }
            Aldrich_Ames_Remix => {
                state.add_effect(Side::USSR, Effect::AldrichAmes);
                let allowed: Vec<_> = state.deck.us_hand().iter().map(|c| *c as usize).collect();
                let d = Decision::new(Side::USSR, Action::Discard, allowed);
                pa!(state, d);
            }
            Pershing_II_Deployed => {
                state.vp -= 1;
                let allowed: Vec<_> = country::WESTERN_EUROPE
                    .iter()
                    .copied()
                    .filter(|x| state.countries[*x].has_influence(Side::US))
                    .collect();
                let d = Decision::with_quantity(Side::USSR, Action::Remove, allowed, 3);
                pa!(state, d);
            }
            Solidarity => state.countries[CName::Poland as usize].us += 3,
            Iran_Iraq_War => {
                let d = Decision::new(side, Action::War, &country::IRAN_IRAQ[..]);
                pa!(state, d);
            }
            Defectors => {
                if *state.side() == Side::USSR && state.ar != 0 {
                    state.vp += 1
                }
            }
            The_Cambridge_Five => {
                state.add_effect(Side::USSR, Effect::US_Scoring_Revealed);
                let scoring = state.deck.scoring_cards(Side::US);
                if !scoring.is_empty() {
                    let mut vec: Vec<usize> = Vec::new();
                    for c in scoring {
                        let region = c.scoring_region().unwrap();
                        vec.extend(&region.all_countries());
                    }
                    let d = Decision::new(Side::US, Action::Place, vec);
                    pa!(state, d);
                }
            }
            Special_Relationship => {
                // Check for UK control when we see if we can even event the card
                if state.has_effect(Side::US, Effect::Nato) {
                    state.vp += 2;
                    let allowed = &country::WESTERN_EUROPE[..];
                    let d = Decision::new(Side::US, Action::Place, allowed);
                    pa!(state, d);
                } else {
                    let allowed = &country::EDGES[CName::UK as usize];
                    let d = Decision::new(Side::US, Action::Place, allowed.clone());
                    pa!(state, d);
                }
            }
            NORAD => state.add_effect(Side::US, Effect::Norad),
            Che => {
                state.set_limit(1);
                let allowed: Vec<_> = country::LATIN_AMERICA[..]
                    .iter()
                    .chain(&country::AFRICA[..])
                    .copied()
                    .filter(|x| {
                        let c = &state.countries[*x];
                        !c.bg && c.has_influence(Side::US)
                    })
                    .collect();
                let ops = self.modified_ops(Side::USSR, state);
                let d = Decision::with_quantity(Side::USSR, Action::Coup, allowed, ops);
                // Get the second coup from the next_decision() API
                pa!(state, d);
            }
            Our_Man_In_Tehran => {
                let mut vec = vec![0];
                vec.extend(state.deck.our_man(rng).iter().map(|c| *c as usize));
                let len = vec.len() as i8;
                let d = Decision::with_quantity(Side::US, Action::ChooseCard, vec, len);
                pa!(state, d);
            }
            Yuri_And_Samantha => state.add_effect(Side::USSR, Effect::Yuri),
            AWACS => {
                state.add_effect(Side::US, Effect::AWACS);
                state.countries[CName::SaudiaArabia as usize].us += 2;
            }
            Olympic_Games
            | Blockade
            | Warsaw_Pact_Formed
            | Junta
            | South_African_Unrest
            | Latin_American_Debt_Crisis
            | Wargames
            | The_China_Card => unimplemented!(),
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
            Flower_Power => !state.has_effect(Side::US, Effect::EvilEmpire),
            One_Small_Step => {
                state.space[eventer as usize] < state.space[eventer.opposite() as usize]
            }
            Wargames => state.defcon() == 2,
            Solidarity => state.has_effect(Side::US, Effect::AllowSolidarity),
            The_Cambridge_Five => state.period() != Period::Late,
            Special_Relationship => state.countries[CName::UK as usize].controller() == Side::US,
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
    pub fn scoring_region(&self) -> Option<Region> {
        match self {
            Card::Africa_Scoring => Some(Region::Africa),
            Card::Asia_Scoring => Some(Region::Asia),
            Card::Central_America_Scoring => Some(Region::CentralAmerica),
            Card::Europe_Scoring => Some(Region::Europe),
            Card::Middle_East_Scoring => Some(Region::MiddleEast),
            Card::South_America_Scoring => Some(Region::SouthAmerica),
            Card::Southeast_Asia_Scoring => Some(Region::SoutheastAsia),
            _ => None,
        }
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
