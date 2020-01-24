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

#[derive(Clone, Copy)]
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
    ]
}
