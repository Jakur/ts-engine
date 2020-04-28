use crate::action::Action;
use crate::agent::ScriptedAgent;
use crate::card::Card;
use crate::country::{CName, Side};
use crate::game::Game;
use crate::state::{DebugRand, GameState};
use crate::tensor::OutputIndex;
use nom::{
    self,
    bytes::complete::{is_a, is_not, tag},
    combinator::{map, opt},
    error::ErrorKind,
    sequence::tuple,
    Err::Error,
    IResult,
};
use std::collections::HashMap;

lazy_static! {
    static ref CARDS: HashMap<String, Card> = {
        (1..Card::total())
            .map(|i| {
                let c = Card::from_index(i);
                (format!("{:?}", c), c)
            })
            .collect()
    };
    static ref COUNTRIES: HashMap<String, CName> = {
        (1..CName::total())
            .map(|i| {
                let c = CName::from_index(i);
                (format!("{:?}", c), c)
            })
            .collect()
    };
}

// Todo real error handling?
macro_rules! become_err {
    ($e:expr) => {
        Err(Error(($e, ErrorKind::Fix)))
    };
}

pub struct Record {
    pub ussr_agent: ScriptedAgent,
    pub us_agent: ScriptedAgent,
    pub rng: DebugRand,
}

impl Into<Game<ScriptedAgent, ScriptedAgent, DebugRand>> for Record {
    fn into(self) -> Game<ScriptedAgent, ScriptedAgent, DebugRand> {
        let state = GameState::new();
        Game::new(self.ussr_agent, self.us_agent, state, self.rng)
    }
}

fn side(x: &str) -> IResult<&str, Side> {
    let p = map(
        nom::branch::alt((tag("USSR"), tag("US"))),
        |s: &str| match s {
            "USSR" => Side::USSR,
            "US" => Side::US,
            _ => unreachable!(),
        },
    );
    p(x)
}

fn space(x: &str) -> IResult<&str, &str> {
    nom::bytes::complete::is_a(" \t")(x)
}

fn card(x: &str) -> IResult<&str, Card> {
    let (left, word) = nom::bytes::complete::is_not(" \t")(x)?;
    if let Some(card) = CARDS.get(word) {
        Ok((left, *card)) // Valid card
    } else {
        become_err![x]
    }
}

#[derive(Debug, PartialEq)]
enum MetaAction {
    Real(Action),
    Touch,
    Roll,
    Unknown,
}

fn action(x: &str) -> IResult<&str, MetaAction> {
    let (left, word) = nom::bytes::complete::is_not(" \t")(x)?;
    // Todo Realignment
    let meta = match word {
        "Inf" => MetaAction::Real(Action::Influence),
        "Coup" => MetaAction::Real(Action::Coup),
        "Place" => MetaAction::Real(Action::Place),
        "Special" => MetaAction::Real(Action::SpecialEvent),
        "Remove" => MetaAction::Real(Action::Remove),
        "Roll" => MetaAction::Roll,
        "HL" => MetaAction::Real(Action::ChooseCard), // Headline
        "OE" => MetaAction::Real(Action::OpsEvent),
        "EO" => MetaAction::Real(Action::EventOps),
        "E" => MetaAction::Real(Action::Event),
        "O" => MetaAction::Real(Action::Ops),
        "War" => MetaAction::Real(Action::War),
        "Discard" => MetaAction::Real(Action::Discard),
        "Space" => MetaAction::Real(Action::Space),
        "Pass" => MetaAction::Real(Action::Pass),
        "Touch" => MetaAction::Touch,
        _ => panic!("Unexpected keyword {}", word),
    };
    Ok((left, meta))
}

fn choices(x: &str) -> IResult<&str, Vec<usize>> {
    let (left, list) = nom::multi::separated_list(tag(" "), is_not(" "))(x)?;
    let mut output = Vec::new();
    for s in list {
        if let Ok(num) = s.parse::<usize>() {
            output.push(num)
        } else {
            // If we have a lone country name, e.g. "Italy"
            if let Some(c) = COUNTRIES.get(s) {
                output.push(*c as usize);
            } else {
                // Check for shorthand, e.g. "3-Italy"
                let expanded = expand_country(s);
                if let Ok(ex) = expanded {
                    output.extend(ex.1.into_iter());
                } else {
                    return become_err![x];
                }
            }
        }
    }
    Ok((left, output))
}

fn expand_country(x: &str) -> IResult<&str, Vec<usize>> {
    let (left, (first, _, country_str)) = tuple((is_not("-"), is_a("-"), is_not(" ")))(x)?;
    let num: usize = match first.parse() {
        Ok(n) => n,
        _ => return become_err![x],
    };
    let country_index = match COUNTRIES.get(country_str) {
        Some(c) => *c as usize,
        _ => return become_err![x],
    };
    let vec: Vec<_> = std::iter::repeat(country_index).take(num).collect();
    Ok((left, vec))
}

#[derive(Debug)]
struct Parsed {
    side: Side,
    card: Option<Card>,
    action: MetaAction,
    choices: Option<Vec<usize>>,
}

fn parse_line(line: &str, last_side: Side) -> Option<Parsed> {
    let (_, (side, _, card, _, act, _, choices)) = tuple((
        opt(side),
        opt(space),
        opt(card),
        opt(space),
        opt(action),
        opt(space),
        opt(choices),
    ))(line)
    .ok()?;
    let side = side.unwrap_or(last_side);
    let act = act.unwrap_or(MetaAction::Unknown);
    dbg!(line);
    let act = if let MetaAction::Unknown = act {
        if let Some(c) = card {
            // Opponent card
            if c.side() == side.opposite() {
                MetaAction::Real(Action::OpsEvent) // Default way of playing opp card
            } else {
                MetaAction::Real(Action::Ops)
            }
        } else {
            unimplemented!()
        }
    } else {
        act
    };
    Some(Parsed {
        side,
        card,
        action: act,
        choices,
    })
}

pub fn parse_lines(string: &str) -> Record {
    let mut last_side = Side::USSR;
    let mut us_rolls = Vec::new();
    let mut ussr_rolls = Vec::new();
    let mut us_cards = Vec::new();
    let mut ussr_cards = Vec::new();
    let mut choices = [Vec::new(), Vec::new()];
    for line in string.lines() {
        if line.starts_with("#") {
            continue;
        }
        if let Some(parsed) = parse_line(line, last_side) {
            last_side = parsed.side;
            match parsed.action {
                MetaAction::Roll => {
                    let roll = parsed.choices.unwrap()[0] as i8;
                    match parsed.side {
                        Side::US => us_rolls.push(roll),
                        Side::USSR => ussr_rolls.push(roll),
                        _ => unimplemented!(),
                    }
                }
                MetaAction::Touch => {
                    let card = parsed.card.expect("Found a card to add");
                    match parsed.side {
                        Side::US => us_cards.push(card),
                        Side::USSR => ussr_cards.push(card),
                        _ => unimplemented!(),
                    }
                }
                MetaAction::Real(act) => match act {
                    Action::Event
                    | Action::EventOps
                    | Action::Ops
                    | Action::OpsEvent
                    | Action::ChooseCard
                    | Action::Discard
                    | Action::Space => {
                        let card = parsed.card.unwrap();
                        // Todo find exceptions to this
                        match parsed.side {
                            Side::US => us_cards.push(card),
                            Side::USSR => ussr_cards.push(card),
                            _ => unimplemented!(),
                        }
                        let x = OutputIndex::encode_single(act, card as usize);
                        choices[parsed.side as usize].push(x);
                    }
                    Action::Influence
                    | Action::Coup
                    | Action::Realignment
                    | Action::Place
                    | Action::Remove
                    | Action::War
                    | Action::SpecialEvent => {
                        for choice in parsed.choices.unwrap() {
                            let x = OutputIndex::encode_single(act, choice);
                            choices[parsed.side as usize].push(x);
                        }
                    }
                    Action::Pass => choices[parsed.side as usize].push(OutputIndex::pass()),
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        } else {
            dbg!(line);
            unimplemented!();
        }
    }
    // Reverse rolls since we remove them LIFO instead of FIFO
    us_rolls = us_rolls.into_iter().rev().collect();
    ussr_rolls = ussr_rolls.into_iter().rev().collect();
    let us_agent = ScriptedAgent::new(&choices[Side::US as usize]);
    let ussr_agent = ScriptedAgent::new(&choices[Side::USSR as usize]);
    // dbg!(&us_agent.choices.lock().unwrap());
    let rng = DebugRand::new(us_rolls, ussr_rolls, vec![], us_cards, ussr_cards);
    Record {
        ussr_agent,
        us_agent,
        rng,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_parse() {
        assert_eq!(side("USSR Place 20 20"), Ok((" Place 20 20", Side::USSR)));
        assert_eq!(
            side("US Warsaw_Pact_Formed OE"),
            Ok((" Warsaw_Pact_Formed OE", Side::US))
        );
        assert!(side("IDK").is_err());
        assert_eq!(
            card("Warsaw_Pact_Formed OE"),
            Ok((" OE", Card::Warsaw_Pact_Formed))
        );
        assert!(card("NotNATO").is_err());
        let (empty, c) = choices("20 40 60").unwrap();
        assert_eq!(empty, "");
        assert_eq!(c, vec![20usize, 40, 60]);
        let (empty, expand) = expand_country("4-Poland").unwrap();
        assert_eq!(empty, "");
        let poland = CName::Poland as usize;
        let eg = CName::EGermany as usize;
        let aust = CName::Austria as usize;
        assert_eq!(expand, vec![poland; 4]);
        assert!(expand_country("6-Beers").is_err());
        let (empty, choices) = choices("4-Poland 2-EGermany 1-Austria").unwrap();
        assert_eq!(empty, "");
        assert_eq!(choices, vec![poland, poland, poland, poland, eg, eg, aust]);
    }
    #[test]
    fn line_parse() {
        let input = "US Warsaw_Pact_Formed OE";
        let (_, (side, _, card, _, act)) =
            tuple((side, space, card, space, action))(input).unwrap();
        dbg!(side);
        dbg!(card);
        dbg!(act);
    }
}
