use crate::card::Card;
use crate::country::{Side, CName};
use crate::action::Action;
use nom::{
    self,
    error::ErrorKind,
    Err::Error,
    IResult,
    bytes::complete::{is_not, is_a, tag},
    combinator::{map, opt},
    sequence::tuple};
use std::collections::HashMap;

lazy_static!{
    static ref CARDS: HashMap<String, Card> = {
        (1..Card::total()).map(|i| {
            let c = Card::from_index(i);
            (format!("{:?}", c), c)
        }).collect()
    };
    static ref COUNTRIES: HashMap<String, CName> = {
        (1..CName::total()).map(|i| {
            let c = CName::from_index(i);
            (format!("{:?}", c), c)
        }).collect()
    };
}

// Todo real error handling?
macro_rules! become_err {
    ($e:expr) => {Err(Error(($e, ErrorKind::Fix)))};
}

fn side(x: &str) -> IResult<&str, Side> {
    let p = map(nom::branch::alt((tag("USSR"), tag("US"))), |s: &str| {
        match s {
            "USSR" => Side::USSR,
            "US" => Side::US,
            _ => unreachable!(),
        }
    });
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

#[derive(Debug)]
enum MetaAction {
    Real(Action),
    Roll,
    OE,
    EO,
    Unknown,
}

fn action(x: &str) -> IResult<&str, MetaAction> {
    let (left, word) = nom::bytes::complete::is_not(" \t")(x)?;
    let meta = match word {
        "Inf" => MetaAction::Real(Action::StandardOps),
        "Place" => MetaAction::Real(Action::Place),
        "Special" => MetaAction::Real(Action::SpecialEvent),
        "Remove" => MetaAction::Real(Action::Remove),
        "Roll" => MetaAction::Roll,
        "OE" => MetaAction::OE,
        "EO" => MetaAction::EO,
        "E" => MetaAction::Real(Action::Event),
        "O" => MetaAction::Real(Action::ConductOps),
        _ => MetaAction::Unknown, // Always a play card ?
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
            let expanded = expand_country(s);
            if let Ok(ex) = expanded {
                output.extend(ex.1.into_iter());
            } else {
                return become_err![x]
            }
        }
    }
    Ok((left, output))
}

fn expand_country(x: &str) -> IResult<&str, Vec<usize>> {
    let (left, (num, _, country_str)) = tuple((is_not("-"), is_a("-"), is_not(" ")))(x)?;
    let num: usize = match num.parse() {
        Ok(n) => n,
        _ => return become_err![x]
    };
    let country_index = match COUNTRIES.get(country_str) {
        Some(c) => *c as usize,
        _ => return become_err![x]
    };
    let vec: Vec<_> = std::iter::repeat(country_index).take(num).collect();
    Ok((left, vec))
}

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
        action, // Not optional ?
        opt(space),
        opt(choices)
    ))(line).ok()?;
    let side = side.unwrap_or(last_side);
    let action = if let MetaAction::Unknown = act {
        if let Some(c) = card {
            // Opponent card
            if c.side().opposite() == side { 
                MetaAction::OE // Default way of playing opp card
            } else {
                MetaAction::Real(Action::ConductOps)
            }
        } else {
            unimplemented!()
        }
    } else {
        act
    };
    Some(Parsed {side, card, action, choices})
}

fn parse_lines(string: &str) {
    let mut last_side = Side::USSR;
    // let mut rolls = [Vec::new(), Vec::new()];
    for line in string.lines() {
        if let Some(parsed) = parse_line(line, last_side) {
            last_side = parsed.side;
        }
    }
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_parse() {
        assert_eq!(side("USSR Place 20 20"), Ok((" Place 20 20", Side::USSR)));
        assert_eq!(side("US Warsaw_Pact_Formed OE"), Ok((" Warsaw_Pact_Formed OE", Side::US)));
        assert!(side("IDK").is_err());
        assert_eq!(card("Warsaw_Pact_Formed OE"), Ok((" OE", Card::Warsaw_Pact_Formed)));
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
        let (_, (side, _, card, _, act)) = tuple((side, space, card, space, action))(input).unwrap();
        dbg!(side);
        dbg!(card);
        dbg!(act);
    }
}