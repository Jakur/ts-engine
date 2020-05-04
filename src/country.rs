use super::name_index;
use crate::card::Effect;
use crate::state::GameState;

use num_traits::FromPrimitive;
use std::collections::HashSet;

pub const NUM_COUNTRIES: usize = CName::USSR as usize + 1;
pub const US_INDEX: usize = CName::US as usize;
pub const USSR_INDEX: usize = CName::USSR as usize;
name_index![INDIA_PAKISTAN; CName::India, CName::Pakistan];
name_index![SUEZ; CName::France, CName::UK, CName::Israel];
name_index![OPEC; CName::Egypt, CName::Iran, CName::Libya, CName::SaudiaArabia, 
    CName::Iraq, CName::GulfStates, CName::Venezuela];
name_index![IND_REDS; CName::Yugoslavia, CName::Romania, CName::Bulgaria, 
    CName::Hungary, CName::Czechoslovakia];

lazy_static! {
    pub static ref EUROPE: Vec<usize> = Region::Europe.all_countries();
    pub static ref ASIA: Vec<usize> = Region::Asia.all_countries();
    pub static ref MIDDLE_EAST: Vec<usize> = Region::MiddleEast.all_countries();
    pub static ref WESTERN_EUROPE: Vec<usize> = Region::WesternEurope.all_countries();
    pub static ref EASTERN_EUROPE: Vec<usize> = Region::EasternEurope.all_countries();
    pub static ref AFRICA: Vec<usize> = Region::Africa.all_countries();
    pub static ref SOUTH_AMERICA: Vec<usize> = Region::SouthAmerica.all_countries();
    pub static ref CENTRAL_AMERICA: Vec<usize> = Region::CentralAmerica.all_countries();
    pub static ref SOUTHEAST_ASIA: Vec<usize> = Region::SoutheastAsia.all_countries();
    pub static ref LATIN_AMERICA: Vec<usize> = {
        let mut v = SOUTH_AMERICA.clone();
        v.extend(CENTRAL_AMERICA.iter().copied());
        v
    };
    pub static ref DECOL: Vec<usize> = AFRICA
        .iter()
        .cloned()
        .chain(SOUTHEAST_ASIA.iter().cloned())
        .collect();
    pub static ref BRUSH_TARGETS: Vec<usize> = countries()
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.stability <= 2 {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    pub static ref EDGES: Vec<Vec<usize>> = adjacency_list();
}

// name_index![OPEC; CName::Egypt, CName::Iran, CName::Libya, CName::SaudiaArabia, CName::Iraq, CName::GulfStates, CName::Venezuela];
// name_index![2, 3, 4];
// name_index![3, 5];
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    US = 0,
    USSR = 1,
    Neutral = 2,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::US => Side::USSR,
            Side::USSR => Side::US,
            Side::Neutral => unimplemented!(), // Not sure if this should be called
        }
    }
}

impl From<usize> for Side {
    fn from(num: usize) -> Self {
        if num == 0 {
            Side::US
        } else if num == 1 {
            Side::USSR
        } else {
            Side::Neutral
        }
    }
}

pub fn access(state: &GameState, side: Side) -> Vec<usize> {
    let mut set = HashSet::new();
    if state.iron_lady && side == Side::USSR {
        let arg = CName::Argentina as usize;
        for (i, list) in EDGES
            .iter()
            .enumerate()
            .filter(|(i, _list)| *i != arg && state.countries[*i].has_influence(side))
        {
            for &v in list.iter() {
                set.insert(v);
            }
            set.insert(i);
        }
    } else {
        for (i, list) in EDGES
            .iter()
            .enumerate()
            .filter(|(i, _list)| state.countries[*i].has_influence(side))
        {
            for &v in list.iter() {
                set.insert(v);
            }
            set.insert(i);
        }
    }
    set.remove(&US_INDEX);
    set.remove(&USSR_INDEX);
    set.into_iter().collect()
}

fn adjacency_list() -> Vec<Vec<usize>> {
    let mut edge_list = vec![Vec::new(); NUM_COUNTRIES];
    let e = edges();
    for (v1, v2) in e.into_iter() {
        edge_list[v1 as usize].push(v2 as usize);
        edge_list[v2 as usize].push(v1 as usize);
    }
    edge_list
}

#[derive(Clone, Copy, PartialEq)]
pub enum Region {
    Europe,
    WesternEurope,
    EasternEurope,
    MiddleEast,
    Asia,
    SoutheastAsia,
    Africa,
    CentralAmerica,
    SouthAmerica,
}

#[derive(Clone, Copy)]
pub enum Status {
    Zero,
    Presence,
    Domination,
    Control,
}

impl Region {
    pub fn major_regions() -> Vec<Region> {
        use Region::*;
        vec![
            Europe,
            MiddleEast,
            Asia,
            Africa,
            CentralAmerica,
            SouthAmerica,
        ]
    }
    /// Returns the status of both sides in the region, and the battleground difference
    pub fn status(&self, state: &GameState, use_shuttle: bool) -> ([Status; 2], i8) {
        let mut ussr_bg = 0;
        let mut ussr_n = 0;
        let mut us_bg = 0;
        let mut us_n = 0;
        for i in self.all_countries() {
            let c = &state.countries[i];
            match c.controller() {
                Side::US => {
                    if c.bg {
                        us_bg += 1;
                    } else {
                        us_n += 1;
                    }
                }
                Side::USSR => {
                    if c.bg {
                        ussr_bg += 1;
                    } else {
                        ussr_n += 1;
                    }
                }
                _ => {}
            }
        }
        let mut ret = [Status::Zero, Status::Zero];
        let bg_total = match self {
            Region::Europe => 5,
            Region::MiddleEast => 6,
            Region::Asia => 6,
            Region::Africa => 5,
            Region::CentralAmerica => 3,
            Region::SouthAmerica => 4,
            _ => 0,
        };
        match self {
            Region::MiddleEast => {
                if state.has_effect(Side::US, Effect::ShuttleDiplomacy) {
                    if !state.is_final_scoring() && use_shuttle {
                        ussr_bg = std::cmp::max(0, ussr_bg - 1);
                    }
                }
            }
            Region::Asia => {
                if state.has_effect(Side::US, Effect::ShuttleDiplomacy) {
                    if !state.is_final_scoring() && use_shuttle {
                        ussr_bg = std::cmp::max(0, ussr_bg - 1);
                    }
                }
                if state.is_controlled(Side::US, CName::Taiwan) {
                    if state.has_effect(Side::US, Effect::FormosanResolution) {
                        us_bg += 1;
                        us_n -= 1;
                    }
                }
            }
            _ => {}
        }
        let diff = us_bg + us_n - ussr_bg - ussr_n;
        let ussr_status = {
            if ussr_bg == bg_total && diff < 0 {
                Status::Control
            } else if ussr_bg > us_bg && diff < 0 && ussr_n > 0 {
                Status::Domination
            } else if ussr_bg + ussr_n > 0 {
                Status::Presence
            } else {
                Status::Zero
            }
        };
        let us_status = {
            if us_bg == bg_total && diff > 0 {
                Status::Control
            } else if us_bg > ussr_bg && diff > 0 && us_n > 0 {
                Status::Domination
            } else if us_bg + us_n > 0 {
                Status::Presence
            } else {
                Status::Zero
            }
        };
        ret[Side::US as usize] = us_status;
        ret[Side::USSR as usize] = ussr_status;
        if let Region::SoutheastAsia = self {
            (ret, us_n - ussr_n + 2 * us_bg - 2 * ussr_bg)
        } else {
            (ret, us_bg - ussr_bg)
        }
    }
    pub fn score(&self, state: &mut GameState) -> i8 {
        use Region::*;
        // Vp for the three successive scoring levels
        let (p, d, c) = match self {
            Europe => (3, 7, 0),
            MiddleEast => (3, 5, 7),
            Asia => (3, 7, 9),
            Africa => (1, 4, 6),
            CentralAmerica => (1, 3, 5),
            SouthAmerica => (2, 5, 6),
            _ => (0, 0, 0),
        };
        let mut vp_change = 0;
        let (statuses, bg_diff) = self.status(state, !state.is_final_scoring());
        // Special case adjacency
        match self {
            Europe => {
                for c in [CName::Finland, CName::Poland, CName::Romania].iter() {
                    if state.is_controlled(Side::US, *c) {
                        vp_change += 1;
                    }
                }
                if state.is_controlled(Side::USSR, CName::Canada) {
                    vp_change -= 1;
                }
            }
            MiddleEast => {
                if let Some(index) = state.effect_pos(Side::US, Effect::ShuttleDiplomacy) {
                    if !state.is_final_scoring() {
                        state.clear_effect(Side::US, index);
                    }
                }
            }
            Asia => {
                for c in [CName::Afghanistan, CName::NKorea].iter() {
                    if state.is_controlled(Side::US, *c) {
                        vp_change += 1;
                    }
                }
                if state.is_controlled(Side::USSR, CName::Japan) {
                    vp_change -= 1;
                }
                if let Some(index) = state.effect_pos(Side::US, Effect::ShuttleDiplomacy) {
                    if !state.is_final_scoring() {
                        state.clear_effect(Side::US, index);
                    }
                }
            }
            CentralAmerica => {
                for c in [CName::Mexico, CName::Cuba].iter() {
                    if state.is_controlled(Side::USSR, *c) {
                        vp_change -= 1;
                    }
                }
            }
            _ => {}
        }
        // Usual scoring protocol
        if let Region::SoutheastAsia = self {
            vp_change += bg_diff;
            state.vp += vp_change;
            return vp_change;
        }
        let us_status = statuses[Side::US as usize];
        let ussr_status = statuses[Side::USSR as usize];
        // Auto win for control
        if *self == Europe {
            if let Status::Control = us_status {
                let x = 20 - state.vp;
                state.vp = 20;
                return x;
            }
            if let Status::Control = ussr_status {
                let x = -20 - state.vp;
                state.vp = -20;
                return x;
            }
        }

        match us_status {
            Status::Presence => vp_change += p,
            Status::Domination => vp_change += d,
            Status::Control => vp_change += c,
            _ => {}
        }
        match ussr_status {
            Status::Presence => vp_change -= p,
            Status::Domination => vp_change -= d,
            Status::Control => vp_change -= c,
            _ => {}
        }
        // 1 point per battleground
        vp_change += bg_diff;
        state.vp += vp_change;
        return vp_change;
    }
    fn low_high(&self) -> (usize, usize) {
        use CName::*;
        match self {
            Region::Europe => (0, Bulgaria as usize),
            Region::WesternEurope => (0, Finland as usize),
            Region::EasternEurope => (Austria as usize, Bulgaria as usize),
            Region::MiddleEast => (Lebanon as usize, SaudiaArabia as usize),
            Region::Asia => (Afghanistan as usize, NKorea as usize),
            Region::SoutheastAsia => (Burma as usize, Philippines as usize),
            Region::Africa => (Morocco as usize, SouthAfrica as usize),
            Region::CentralAmerica => (Mexico as usize, DominicanRep as usize),
            Region::SouthAmerica => (Venezuela as usize, Uruguay as usize),
        }
    }
    pub fn has_country(&self, index: usize) -> bool {
        let (low, high) = self.low_high();
        low <= index && index <= high
    }
    pub fn all_countries(&self) -> Vec<usize> {
        let (low, high) = self.low_high();
        (low..=high).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Country {
    pub stability: i8,
    pub us: i8,
    pub ussr: i8,
    pub bg: bool,
}

impl Country {
    pub fn controller(&self) -> Side {
        let diff = self.us - self.ussr;
        if diff >= self.stability {
            Side::US
        } else if diff <= -1 * self.stability {
            Side::USSR
        } else {
            Side::Neutral
        }
    }
    pub fn has_influence(&self, side: Side) -> bool {
        match side {
            Side::US => self.us > 0,
            Side::USSR => self.ussr > 0,
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn influence(&self, side: Side) -> i8 {
        match side {
            Side::US => self.us,
            Side::USSR => self.ussr,
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn greater_influence(&self) -> Side {
        if self.ussr > self.us {
            Side::USSR
        } else if self.us > self.ussr {
            Side::US
        } else {
            Side::Neutral
        }
    }
    fn new_bg(stability: i8) -> Country {
        Country {
            stability,
            us: 0,
            ussr: 0,
            bg: true,
        }
    }
    fn new_non(stability: i8) -> Country {
        Country {
            stability,
            us: 0,
            ussr: 0,
            bg: false,
        }
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum CName {
    Turkey = 0,
    Greece,
    Italy,
    SpainPortugal,
    France,
    WGermany,
    Benelux,
    UK,
    Canada,
    Norway,
    Denmark,
    Sweden,
    Austria, // Start East
    Finland, // End West
    EGermany,
    Poland,
    Czechoslovakia,
    Hungary,
    Romania,
    Yugoslavia,
    Bulgaria, // End Europe
    Lebanon,
    Syria,
    Israel,
    Iraq,
    Iran,
    Libya,
    Egypt,
    Jordan,
    GulfStates,
    SaudiaArabia,
    Afghanistan,
    Pakistan,
    India,
    Burma,
    LaosCambodia,
    Thailand,
    Vietnam,
    Malaysia,
    Indonesia,
    Philippines,
    Australia,
    Taiwan,
    Japan,
    SKorea,
    NKorea,
    Morocco,
    Algeria,
    Tunisia,
    WestAfricanStates,
    SaharanStates,
    Sudan,
    IvoryCoast,
    Nigeria,
    Ethiopia,
    Somalia,
    Cameroon,
    Zaire,
    Kenya,
    Angola,
    SEAfricanStates,
    Zimbabwe,
    Botswana,
    SouthAfrica,
    Mexico,
    Guatemala,
    ElSalvador,
    Honduras,
    CostaRica,
    Panama,
    Nicaragua,
    Cuba,
    Haiti,
    DominicanRep,
    Venezuela,
    Colombia,
    Ecuador,
    Peru,
    Brazil,
    Bolivia,
    Chile,
    Paraguay,
    Argentina,
    Uruguay,
    US,
    USSR,
}

impl From<CName> for usize {
    fn from(item: CName) -> Self {
        item as usize
    }
}

impl CName {
    pub fn from_index(index: usize) -> CName {
        CName::from_usize(index).unwrap()
    }
    pub fn total() -> usize {
        NUM_COUNTRIES
    }
}
/// Returns countries with their starting influence before players have any agency.
pub fn standard_start() -> Vec<Country> {
    use CName::*;
    let mut c = countries();
    let ussr = [
        (1, Syria),
        (1, Iraq),
        (3, NKorea),
        (3, EGermany),
        (1, Finland),
        (6, USSR),
    ];
    let us = [
        (2, Canada),
        (1, Iran),
        (1, Israel),
        (1, Japan),
        (4, Australia),
        (1, Philippines),
        (1, SKorea),
        (1, Panama),
        (1, SouthAfrica),
        (5, UK),
        (6, US),
    ];
    for (x, y) in ussr.iter() {
        c[*y as usize].ussr += x;
    }
    for (x, y) in us.iter() {
        c[*y as usize].us += x;
    }
    c
}

pub fn countries() -> Vec<Country> {
    use CName::*;
    let mut countries = vec![Country::new_non(0); NUM_COUNTRIES];
    let bgs = [
        (France, 3),
        (WGermany, 4),
        (EGermany, 3),
        (Poland, 3),
        (Italy, 2),
        (Libya, 2),
        (Egypt, 2),
        (Israel, 4),
        (SaudiaArabia, 3),
        (Iraq, 3),
        (Iran, 2),
        (Pakistan, 2),
        (India, 3),
        (Thailand, 2),
        (NKorea, 3),
        (SKorea, 3),
        (Japan, 4),
        (Algeria, 2),
        (Nigeria, 1),
        (Zaire, 1),
        (Angola, 1),
        (SouthAfrica, 3),
        (Mexico, 2),
        (Cuba, 3),
        (Panama, 2),
        (Venezuela, 2),
        (Brazil, 2),
        (Chile, 3),
        (Argentina, 2),
    ];
    let non = [
        (Canada, 4),
        (UK, 5),
        (Norway, 4),
        (Denmark, 3),
        (Sweden, 4),
        (Finland, 4),
        (Benelux, 3),
        (Czechoslovakia, 3),
        (Austria, 4),
        (Hungary, 3),
        (Romania, 3),
        (Yugoslavia, 3),
        (Bulgaria, 3),
        (SpainPortugal, 2),
        (Greece, 2),
        (Turkey, 2),
        (Lebanon, 1),
        (Syria, 2),
        (Jordan, 2),
        (GulfStates, 3),
        (Afghanistan, 2),
        (Burma, 2),
        (LaosCambodia, 1),
        (Vietnam, 1),
        (Malaysia, 2),
        (Australia, 4),
        (Indonesia, 1),
        (Philippines, 2),
        (Taiwan, 3),
        (Tunisia, 2),
        (Morocco, 3),
        (WestAfricanStates, 2),
        (IvoryCoast, 2),
        (SaharanStates, 1),
        (Cameroon, 1),
        (Botswana, 2),
        (Zimbabwe, 1),
        (SEAfricanStates, 1),
        (Kenya, 2),
        (Somalia, 2),
        (Ethiopia, 1),
        (Sudan, 1),
        (Guatemala, 1),
        (ElSalvador, 1),
        (Honduras, 2),
        (CostaRica, 3),
        (Nicaragua, 1),
        (Haiti, 1),
        (DominicanRep, 1),
        (Colombia, 1),
        (Ecuador, 2),
        (Peru, 2),
        (Bolivia, 2),
        (Paraguay, 2),
        (Uruguay, 2),
        (US, 6),
        (USSR, 6),
    ];
    for (n, s) in bgs.iter() {
        let c = Country::new_bg(*s);
        countries[*n as usize] = c;
    }
    for (n, s) in non.iter() {
        let c = Country::new_non(*s);
        countries[*n as usize] = c;
    }
    countries
}

fn edges() -> Vec<(CName, CName)> {
    use CName::*;
    vec![
        (US, Canada),
        (Canada, UK),
        (UK, France),
        (UK, Norway),
        (UK, Benelux),
        (Norway, Sweden),
        (Sweden, Denmark),
        (Sweden, Finland),
        (Finland, USSR),
        (France, WGermany),
        (France, Italy),
        (France, SpainPortugal),
        (France, Algeria),
        (WGermany, Benelux),
        (WGermany, Denmark),
        (WGermany, Austria),
        (WGermany, EGermany),
        (EGermany, Poland),
        (EGermany, Czechoslovakia),
        (Poland, USSR),
        (Poland, Czechoslovakia),
        (Czechoslovakia, Hungary),
        (Austria, Italy),
        (Austria, Hungary),
        (Hungary, Yugoslavia),
        (Hungary, Romania),
        (Romania, USSR),
        (Romania, Turkey),
        (Romania, Yugoslavia),
        (Yugoslavia, Italy),
        (Yugoslavia, Greece),
        (Greece, Italy),
        (Greece, Bulgaria),
        (Greece, Turkey),
        (Bulgaria, Turkey),
        (Turkey, Syria),
        (Syria, Lebanon),
        (Syria, Israel),
        (Lebanon, Israel),
        (Lebanon, Jordan),
        (Israel, Egypt),
        (Israel, Jordan),
        (Egypt, Libya),
        (Egypt, Sudan),
        (Libya, Tunisia),
        (Jordan, Iraq),
        (Jordan, SaudiaArabia),
        (Iraq, SaudiaArabia),
        (Iraq, GulfStates),
        (Iraq, Iran),
        (SaudiaArabia, GulfStates),
        (Iran, Afghanistan),
        (Iran, Pakistan),
        (Afghanistan, USSR),
        (Afghanistan, Pakistan),
        (Pakistan, India),
        (India, Burma),
        (Burma, LaosCambodia),
        (LaosCambodia, Thailand),
        (LaosCambodia, Vietnam),
        (Thailand, Vietnam),
        (Thailand, Malaysia),
        (Malaysia, Australia),
        (Malaysia, Indonesia),
        (Indonesia, Philippines),
        (Philippines, Japan),
        (Japan, US),
        (Japan, Taiwan),
        (Japan, SKorea),
        (Taiwan, SKorea),
        (SKorea, NKorea),
        (NKorea, USSR),
        (SpainPortugal, Morocco),
        (SpainPortugal, Italy),
        (Morocco, Algeria),
        (Algeria, Tunisia),
        (Algeria, SaharanStates),
        (Morocco, WestAfricanStates),
        (WestAfricanStates, IvoryCoast),
        (IvoryCoast, Nigeria),
        (SaharanStates, Nigeria),
        (Nigeria, Cameroon),
        (Cameroon, Zaire),
        (Zaire, Angola),
        (Zaire, Zimbabwe),
        (Angola, Botswana),
        (Angola, SouthAfrica),
        (SouthAfrica, Botswana),
        (Botswana, Zimbabwe),
        (Zimbabwe, SEAfricanStates),
        (SEAfricanStates, Kenya),
        (Kenya, Somalia),
        (Somalia, Ethiopia),
        (Sudan, Ethiopia),
        (US, Mexico),
        (US, Cuba),
        (Mexico, Guatemala),
        (Guatemala, ElSalvador),
        (ElSalvador, Honduras),
        (Guatemala, Honduras),
        (Honduras, CostaRica),
        (Honduras, Nicaragua),
        (Nicaragua, Cuba),
        (Cuba, Haiti),
        (Haiti, DominicanRep),
        (CostaRica, Panama),
        (Panama, Colombia),
        (Colombia, Venezuela),
        (Colombia, Ecuador),
        (Venezuela, Brazil),
        (Brazil, Uruguay),
        (Uruguay, Paraguay),
        (Uruguay, Argentina),
        (Paraguay, Argentina),
        (Paraguay, Bolivia),
        (Bolivia, Peru),
        (Argentina, Chile),
        (Chile, Peru),
        (Peru, Ecuador),
        (EGermany, Austria),
        (Nicaragua, CostaRica),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_countries() {
        let countries = countries();
        assert!(countries.into_iter().all(|c| c.stability > 0));
    }
    #[test]
    fn check_east_west_europe() {
        use crate::country::CName::*;
        let west = &[
            Canada,
            UK,
            SpainPortugal,
            France,
            Benelux,
            WGermany,
            Italy,
            Austria,
            Greece,
            Turkey,
            Norway,
            Denmark,
            Sweden,
            Finland,
        ];
        for w in west.into_iter().map(|&x| x as usize) {
            assert!(Region::WesternEurope.has_country(w));
        }
        let east = &[
            Finland,
            EGermany,
            Poland,
            Czechoslovakia,
            Austria,
            Hungary,
            Romania,
            Yugoslavia,
            Bulgaria,
        ];
        for e in east.into_iter().map(|&x| x as usize) {
            assert!(Region::EasternEurope.has_country(e));
        }
    }
    #[test]
    fn check_degrees() {
        use CName::*;
        let e = edges();
        let len = NUM_COUNTRIES;
        let mut edge_list = vec![Vec::new(); len];
        for (v1, v2) in e {
            edge_list[v1 as usize].push(v2);
            edge_list[v2 as usize].push(v1);
        }
        let correct = [
            (Canada, 2),
            (UK, 4),
            (Norway, 2),
            (Sweden, 3),
            (Finland, 2),
            (Denmark, 2),
            (Benelux, 2),
            (France, 5),
            (SpainPortugal, 3),
            (Italy, 5),
            (WGermany, 5),
            (EGermany, 4),
            (Austria, 4),
            (Poland, 3),
            (Czechoslovakia, 3),
            (Hungary, 4),
            (Romania, 4),
            (Yugoslavia, 4),
            (Greece, 4),
            (Bulgaria, 2),
            (Turkey, 4),
            (Syria, 3),
            (Lebanon, 3),
            (Israel, 4),
            (Egypt, 3),
            (Libya, 2),
            (Jordan, 4),
            (Iraq, 4),
            (SaudiaArabia, 3),
            (GulfStates, 2),
            (Iran, 3),
            (Afghanistan, 3),
            (Pakistan, 3),
            (India, 2),
            (Burma, 2),
            (LaosCambodia, 3),
            (Vietnam, 2),
            (Thailand, 3),
            (Malaysia, 3),
            (Australia, 1),
            (Indonesia, 2),
            (Philippines, 2),
            (Japan, 4),
            (Taiwan, 2),
            (SKorea, 3),
            (NKorea, 2),
            (Tunisia, 2),
            (Algeria, 4),
            (Morocco, 3),
            (WestAfricanStates, 2),
            (IvoryCoast, 2),
            (SaharanStates, 2),
            (Nigeria, 3),
            (Cameroon, 2),
            (Zaire, 3),
            (Angola, 3),
            (SouthAfrica, 2),
            (Botswana, 3),
            (Zimbabwe, 3),
            (SEAfricanStates, 2),
            (Kenya, 2),
            (Somalia, 2),
            (Ethiopia, 2),
            (Sudan, 2),
            (Mexico, 2),
            (Guatemala, 3),
            (ElSalvador, 2),
            (Honduras, 4),
            (CostaRica, 3),
            (Panama, 2),
            (Nicaragua, 3),
            (Cuba, 3),
            (Haiti, 2),
            (DominicanRep, 1),
            (Colombia, 3),
            (Venezuela, 2),
            (Brazil, 2),
            (Uruguay, 3),
            (Paraguay, 3),
            (Bolivia, 2),
            (Argentina, 3),
            (Chile, 2),
            (Peru, 3),
            (Ecuador, 2),
            (US, 4),
            (USSR, 5),
        ];
        // Check that I didn't miss a country in this list
        assert_eq!(len, correct.len());
        let s: usize = correct.iter().map(|(x, _y)| *x as usize).sum();
        assert_eq!(s, len * (len - 1) / 2);
        // Check degrees of every node
        assert!(correct
            .iter()
            .all(|(x, y)| edge_list[*x as usize].len() == *y));
    }
}
