use std::fs::File;
use std::io::prelude::*;
use ts_engine;

fn load_file(name: &str) -> String {
    let mut string = String::new();
    let mut f = File::open(name).unwrap();
    f.read_to_string(&mut string).unwrap();
    string
}

#[test]
fn parse_one_turn() {
    use ts_engine::country::{
        self, countries,
        CName::{self, *},
    };
    let s = load_file("tests/Brashers_Ziemovit2020.record");
    let mut game = ts_engine::record::parse_lines(&s);
    assert_eq!(game.rng.us_rolls, vec![6, 1]);
    game.setup();
    // dbg!(&game.rng.us_draw);
    // dbg!(&game.rng.ussr_draw);
    game.play(1, None);
    let us = [
        (Canada, 2i8),
        (UK, 3),
        (France, 1),
        (WGermany, 4),
        (Lebanon, 1),
        (Libya, 1),
        (Egypt, 2),
        (Iraq, 1),
        (Iran, 2),
        (SKorea, 1),
        (Japan, 1),
        (LaosCambodia, 1),
        (Thailand, 3),
        (Malaysia, 1),
        (Australia, 4),
        (SouthAfrica, 1),
        (Panama, 1),
    ];
    let ussr = [
        (Finland, 1i8),
        (Poland, 4),
        (EGermany, 4),
        (Austria, 1),
        (Italy, 3),
        (SpainPortugal, 2),
        (Syria, 2),
        (NKorea, 3),
        (India, 1),
        (Pakistan, 3),
        (Afghanistan, 2),
        (Cuba, 3),
    ];
    let mut all_countries = countries();
    for (c, inf) in us.iter() {
        all_countries[*c as usize].us = *inf;
    }
    for (c, inf) in ussr.iter() {
        all_countries[*c as usize].ussr = *inf;
    }
    // Exclude USSR and US
    for index in 0..country::NUM_COUNTRIES - 2 {
        let c1 = &all_countries[index];
        let c2 = &game.state.countries[index];
        dbg!(CName::from_index(index));
        assert_eq!(c1.us, c2.us);
        assert_eq!(c1.ussr, c2.ussr);
    }
}
