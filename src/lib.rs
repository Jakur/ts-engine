#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate num_derive;

pub mod action;
pub mod agent;
pub mod card;
pub mod country;
pub mod game;
pub mod state;
mod tensor;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scoring() {
        use country::CName::*;
        use country::{Region, Side};
        let mut state = state::GameState::new();
        let us = [
            UK,
            WGermany,
            Poland,
            SpainPortugal,
            Greece,
            Turkey,
            Lebanon,
            Egypt,
            Iran,
            Pakistan,
            India,
            Burma,
            Australia,
            Taiwan,
            NKorea,
            Japan,
            Nigeria,
            Zaire,
            SEAfricanStates,
            Ethiopia,
            Mexico,
            Venezuela,
            Brazil,
            Paraguay,
            Argentina,
        ];
        let ussr = [
            Finland,
            EGermany,
            Czechoslovakia,
            Romania,
            Italy,
            France,
            Syria,
            Iraq,
            SaudiaArabia,
            Libya,
            LaosCambodia,
            Thailand,
            Vietnam,
            Indonesia,
            SKorea,
            Algeria,
            Angola,
            SouthAfrica,
            Cuba,
            Haiti,
            Nicaragua,
            Panama,
            Chile,
            Uruguay,
        ];
        for c in us.into_iter() {
            state.control(Side::US, *c);
        }
        for c in ussr.into_iter() {
            state.control(Side::USSR, *c);
        }
        // Use two copies of Shuttle, so order of scoring doesn't matter
        state.us_effects.push(card::Effect::ShuttleDiplomacy);
        state.us_effects.push(card::Effect::ShuttleDiplomacy);

        state.us_effects.push(card::Effect::FormosanResolution);
        let scores = [
            (Region::Europe, 0),
            (Region::MiddleEast, 0),
            (Region::Asia, 9),
            (Region::Africa, -1),
            (Region::SouthAmerica, 5),
            (Region::CentralAmerica, -4),
            (Region::SoutheastAsia, -4),
            (Region::MiddleEast, -3), // Without Shuttle
            (Region::Asia, 8),        // Without Shuttle
        ];
        for (r, delta) in scores.into_iter() {
            assert_eq!(Region::score(r, &mut state), *delta);
        }
    }
}
