#![allow(non_camel_case_types)]

use crate::action::{self, Action, Decision, EventTime};
use crate::country::{self, CName, Country, Region, Side, Status};
use crate::state::{GameState, Period, TwilightRand};

use num_traits::FromPrimitive;

pub mod deck;
pub mod effect;
pub use deck::*;
pub use effect::*;

const NUM_CARDS: usize = Card::Willy_Brandt as usize + 1;

const IND_REDS: [usize; 5] = [
    CName::Yugoslavia as usize,
    CName::Romania as usize,
    CName::Bulgaria as usize,
    CName::Hungary as usize,
    CName::Czechoslovakia as usize,
];

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
    Brush_War,
    Central_America_Scoring,
    Southeast_Asia_Scoring,
    Arms_Race,
    Cuban_Missile_Crisis = 40,
    Nuclear_Subs,
    Quagmire,
    SALT_Negotiations,
    Bear_Trap,
    Summit,
    How_I_Learned_To_Stop_Worrying, // I'm sorry this is so long
    Junta,
    Kitchen_Debates,
    Missile_Envy,
    We_Will_Bury_You = 50,
    Brezhnev_Doctrine,
    Portuguese_Empire_Crumbles,
    South_African_Unrest,
    Allende,
    Willy_Brandt,
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
            Side::Neutral => *state.side(),
        };
        if !self.can_event(state) {
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
                    // pa!(state, d);
                }
            }
            Warsaw_Pact_Formed => {
                if !state.us_effects.contains(&Effect::AllowNato) {
                    state.us_effects.push(Effect::AllowNato);
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
            Side::Neutral => *state.side(),
        };
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
        if !self.can_event(state) {
            return false;
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
                        // Todo find out of the US really has agency in these decisions?
                        // Just Chernobyl?
                        let d = if card.is_special() {
                            Decision::new(
                                Side::US,
                                Action::SpecialEvent,
                                card.e_choices(state).unwrap_or_else(|| vec![]),
                            )
                        } else {
                            Decision::new(Side::US, Action::Event, &[])
                        };
                        state.current_event = Some(card);
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
            Vietnam_Revolts => state.ussr_effects.push(Effect::VietnamRevolts),
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
                state.ussr_effects.push(Effect::DeGaulle);
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
            NATO => {
                state.us_effects.push(Effect::Nato);
            }
            Independent_Reds => {
                let allowed = opp_has_inf(&IND_REDS, Side::US, state);
                let x = Decision::new(Side::US, Action::Place, allowed);
                pa!(state, x);
            }
            Marshall_Plan => {
                if !state.us_effects.contains(&Effect::AllowNato) {
                    state.us_effects.push(Effect::AllowNato);
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
            Containment => state.us_effects.push(Effect::Containment),
            CIA_Created => {
                let offset = std::cmp::max(0, state.base_ops_offset(Side::US));
                state.us_effects.push(Effect::USSR_Hand_Revealed);
                pa!(state, Decision::conduct_ops(Side::US, 1 + offset));
            }
            US_Japan_Mutual_Defense_Pact => {
                state.control(Side::US, CName::Japan);
                // This effect is so useless I wonder if I should bother
                state.us_effects.push(Effect::US_Japan);
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
            Formosan_Resolution => state.us_effects.push(Effect::FormosanResolution),
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
                state.us_effects.push(Effect::WWBY);
            }
            Kitchen_Debates => state.vp += 2,
            Brezhnev_Doctrine => state.ussr_effects.push(Effect::Brezhnev),
            Portuguese_Empire_Crumbles => {
                state.countries[CName::Angola as usize].ussr += 2;
                state.countries[CName::SEAfricanStates as usize].ussr += 2;
            }
            Allende => state.countries[CName::Chile as usize].ussr += 2,
            Willy_Brandt => {
                state.vp -= 1;
                state.countries[CName::WGermany as usize].ussr += 1;
                state.ussr_effects.push(Effect::WillyBrandt);
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
    pub fn can_event(&self, state: &GameState) -> bool {
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
        let atts = init_cards();
        assert_eq!(atts.len(), NUM_CARDS);
        let cards: Vec<_> = (1..NUM_CARDS).map(|x| Card::from_u8(x as u8)).collect();
        for c in cards {
            assert!(c.is_some());
        }
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
