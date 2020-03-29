use std::io::prelude::*;
use std::fs::File;
use ts_engine;

fn load_file(name: &str) -> String {
    let mut string = String::new();
    let mut f = File::open(name).unwrap();
    f.read_to_string(&mut string).unwrap();
    string
}

#[test]
fn parse_one_turn() {
    let s = load_file("tests/Brashers_Ziemovit2020.record");
    let mut game = ts_engine::record::parse_lines(&s);
    game.setup();
    // dbg!(&game.rng.us_draw);
    // dbg!(&game.rng.ussr_draw);
    game.play(1, None);
}