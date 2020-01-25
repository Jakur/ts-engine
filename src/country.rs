#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    US,
    USSR,
}

pub struct Map {
    pub countries: Vec<Country>,
    edges: Vec<Vec<usize>>,
}

impl Map {
    pub fn new() -> Map {
        unimplemented!()
    }
}

#[derive(Clone, Copy)]
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

impl Region {
    pub fn score(&self, map: &Vec<Country>) -> i32 {
        // Todo effects, e.g. Formosan and Shuttle
        let countries = self.all_countries();
        let mut ussr_bg = 0;
        let mut ussr_n = 0;
        let mut us_bg = 0;
        let mut us_n = 0;
        for i in countries {
            let c = &map[i];
            match c.controller() {
                Some(x) if x == Side::US => {
                    if c.bg {
                        us_bg += 1;
                    } else {
                        us_n += 1;
                    }
                }
                Some(x) if x == Side::USSR => {
                    if c.bg {
                        ussr_bg += 1;
                    } else {
                        ussr_n += 1;
                    }
                }
                _ => {}
            }
        }
        todo!()
    }
    pub fn all_countries(&self) -> Vec<usize> {
        use CName::*;
        match self {
            Region::Europe => (0..=Finland as usize).collect(),
            Region::WesternEurope => {
                let x = [
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
                x.into_iter().map(|n| *n as usize).collect()
            }
            Region::EasternEurope => {
                let x = [
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
                x.into_iter().map(|n| *n as usize).collect()
            }
            Region::MiddleEast => (Lebanon as usize..=SaudiaArabia as usize).collect(),
            Region::Asia => (Afghanistan as usize..=NKorea as usize).collect(),
            Region::SoutheastAsia => (Burma as usize..=Philippines as usize).collect(),
            Region::Africa => (Morocco as usize..=SouthAfrica as usize).collect(),
            Region::CentralAmerica => (Mexico as usize..=DominicanRep as usize).collect(),
            Region::SouthAmerica => (Venezuela as usize..=Uruguay as usize).collect(),
        }
    }
}

pub struct Country {
    pub stability: i8,
    pub us: i8,
    pub ussr: i8,
    pub bg: bool,
}

impl Country {
    pub fn controller(&self) -> Option<Side> {
        let diff = self.us - self.ussr;
        if diff >= self.stability {
            Some(Side::US)
        } else if diff <= -1 * self.stability {
            Some(Side::USSR)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CName {
    Canada = 0,
    UK,
    France,
    SpainPortugal,
    Benelux,
    Norway,
    Denmark,
    Sweden,
    WGermany,
    EGermany,
    Italy,
    Austria,
    Poland,
    Czechoslovakia,
    Hungary,
    Yugoslavia,
    Greece,
    Romania,
    Bulgaria,
    Turkey,
    Finland,
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
    fn check_degrees() {
        use CName::*;
        let e = edges();
        let len = USSR as usize + 1;
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
