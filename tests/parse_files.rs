use std::fs::File;
use std::io::prelude::*;
use ts_engine;
use ts_engine::card::Card;
use ts_engine::country::{self, CName};
use ts_engine::game::Game;
use ts_engine::record::*;
use ts_engine::state::GameState;

pub fn load_file(name: &str) -> String {
    let mut string = String::new();
    let mut f = File::open(name).unwrap();
    f.read_to_string(&mut string).unwrap();
    string
}

fn check_countries(state: &GameState, us: &[(CName, i8)], ussr: &[(CName, i8)]) {
    let mut all_countries = country::countries();
    for (c, inf) in us.iter() {
        all_countries[*c as usize].us = *inf;
    }
    for (c, inf) in ussr.iter() {
        all_countries[*c as usize].ussr = *inf;
    }
    // Exclude USSR and US
    for index in 0..country::NUM_COUNTRIES - 2 {
        let expected = &all_countries[index];
        let game = &state.countries[index];
        dbg!(CName::from_index(index));
        assert_eq!(expected.us, game.us);
        assert_eq!(expected.ussr, game.ussr);
    }
}

#[test]
fn test_traps() {
    let text = load_file("tests/Traps1.record");
    let record = parse_lines(&text);
    let Record {
        ussr_agent,
        us_agent,
        rng,
    } = record;
    let mut game = Game::turn_one(ussr_agent, us_agent, rng);
    game.state.deck.add_mid_war();
    game.draw_hands();
    dbg!(game.state.deck.us_hand());
    dbg!(game.state.deck.ussr_hand());
    assert!(game.play(1, Some(6)).is_ok());
    dbg!(game.state.deck.removed());
    dbg!(game.state.deck.discard_pile());
    dbg!(game.state.deck.us_hand());
    // Todo test ops in the headline for USSR
}

#[test]
fn events1() {
    use ts_engine::country::{CName::*, Side};
    let s = load_file("tests/Events1.record");
    let mut game: Game<_, _, _> = ts_engine::record::parse_lines(&s).into();
    game.setup();
    dbg!(game.state.turn);
    let res = game.play(1, None);
    assert!(res.is_ok());
    let us = [
        (Canada, 3),
        (UK, 4),
        (SpainPortugal, 1),
        (Italy, 5),
        (WGermany, 5),
        (Greece, 1),
        (Iran, 2),
        (SKorea, 2),
        (Japan, 4),
        (Philippines, 1),
        (Australia, 4),
        (SouthAfrica, 1),
    ];
    let ussr = [
        (EGermany, 5),
        (Poland, 5),
        (Austria, 4),
        (Yugoslavia, 3),
        (Hungary, 1),
        (Finland, 1),
        (France, 1),
        (Egypt, 2),
        (Syria, 1),
        (Iraq, 1),
        (NKorea, 3),
        (Burma, 1),
        (Malaysia, 1),
        (Angola, 1),
        (SouthAfrica, 1),
    ];
    check_countries(&game.state, &us, &ussr);
    // dbg!(game.state.ar);
    assert_eq!(game.state.turn, 2);
    // assert_eq!(game.state.defcon, 4);
    // assert_eq!(game.state.vp, 0);
}

#[test]
fn parse_one_turn() {
    use ts_engine::card::Card;
    use ts_engine::country::{CName::*, Side};
    let s = load_file("tests/Brashers_Ziemovit2020.record");
    let mut game: Game<_, _, _> = ts_engine::record::parse_lines(&s).into();
    assert_eq!(game.rng.us_rolls, vec![6, 1]);
    game.setup();
    assert!(game.play(1, None).is_ok());
    let us = [
        (Canada, 2),
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
        (Philippines, 1),
        (SouthAfrica, 1),
        (Panama, 1),
    ];
    let ussr = [
        (Finland, 1),
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
    dbg!(&game.state.countries[CName::Iraq as usize]);
    check_countries(&game.state, &us, &ussr);
    assert_eq!(game.state.turn, 2);
    assert_eq!(game.state.defcon(), 3);
    assert_eq!(game.state.vp, 0);
    assert!(game.state.us_effects.is_empty());
    assert!(game.state.ussr_effects.is_empty());
    assert_eq!(game.state.space[Side::US as usize], 1);
    assert_eq!(game.state.space[Side::USSR as usize], 0);
    assert_eq!(game.state.mil_ops[Side::US as usize], 0);
    assert_eq!(game.state.mil_ops[Side::USSR as usize], 0);
    assert_eq!(
        game.state.deck.removed(),
        &vec![
            Card::Captured_Nazi_Scientist,
            Card::Suez_Crisis,
            Card::Fidel,
            Card::Independent_Reds,
        ]
    );
    assert!(game.state.deck.china_available(Side::US));
    assert_eq!(
        game.state.deck.discard_pile(),
        &vec![
            Card::Olympic_Games,
            Card::NATO,
            Card::Warsaw_Pact_Formed,
            Card::Arab_Israeli_War,
            Card::Formosan_Resolution,
            Card::Korean_War,
            Card::UN_Intervention,
            Card::Indo_Pakistani_War,
            Card::East_European_Unrest
        ]
    );
}
