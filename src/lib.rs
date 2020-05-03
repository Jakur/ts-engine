#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate num_derive;

pub mod action;
pub mod agent;
pub mod card;
pub mod country;
pub mod game;
pub mod record;
pub mod state;
mod tensor;

#[macro_export]
#[doc(hidden)]
macro_rules! name_index {
    ($name:ident; $($element:expr),*) => {
        pub const $name: [usize; $crate::count![@COUNT; $($element),*]] = [
            $($element as usize),*
        ];
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! count {
    (@COUNT; $($element:expr),*) => {
        <[()]>::len(&[$($crate::count![@SUBST; $element]),*])
    };
    (@SUBST; $_element:expr) => { () };
}

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
        for c in us.iter() {
            state.control(Side::US, *c);
        }
        for c in ussr.iter() {
            state.control(Side::USSR, *c);
        }
        // Use two copies of Shuttle, so order of scoring doesn't matter
        state.add_effect(Side::US, card::Effect::ShuttleDiplomacy);
        state.add_effect(Side::US, card::Effect::ShuttleDiplomacy);

        state.add_effect(Side::US, card::Effect::FormosanResolution);
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
        for (r, delta) in scores.iter() {
            assert_eq!(Region::score(r, &mut state), *delta);
        }
    }
}
