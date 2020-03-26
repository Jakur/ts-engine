use crate::card::Card;
use crate::country::{Side, CName};
use nom::{
    self,
    error::{ErrorKind},
    Err::Error,
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    sequence::tuple};

use std::collections::HashMap;

lazy_static!{
    static ref CARDS: HashMap<String, Card> = {
        (1..Card::total()).map(|i| {
            let c = Card::from_index(i);
            (format!("{:?}", c), c)
        }).collect()
    };
}

fn intermediate(input: String) -> Vec<String> {
    let countries: HashMap<_, _> = (0..CName::total() - 2).map(|i| {
        let c = CName::from_index(i);
        let string = format!("{:?}", c);
        (string, (c as usize).to_string())
    }).collect();
    let us = format!("{:?} ", Side::US);
    let ussr = format!("{:?} ", Side::USSR);
    let mut side = Side::USSR;
    let vec: Vec<_> = input.lines().filter_map(|line| {
        if line.starts_with("#") {
            None
        } else {
            let mut words: Vec<&str> = line.split_whitespace().collect();
            if !(words[0] == "US" || words[0] == "USSR") {
                match side {
                    Side::US => words.insert(0, &us),
                    Side::USSR => words.insert(0, &ussr),
                    _ => unimplemented!(),
                }
            } else {
                if words[0] == "US" {
                    side = Side::US;
                } else if words[0] == "USSR" {
                    side = Side::USSR;
                } else {
                    unimplemented!();
                }
            }
            let country: HashMap<_, (usize, _)> = words.iter().filter_map(|w| {
                if w.contains("-") {
                    let mut iter = w.split("-");
                    let quantity = iter.next().unwrap();
                    let country = iter.next().unwrap();
                    Some((w.clone(), (quantity.parse().unwrap(), country)))
                } else {
                    None
                }
            }).collect();
            words = words.into_iter().filter(|x| country.contains_key(x)).collect();
            for (quantity, country) in country.values() {
                for _ in 0..*quantity {
                    words.push(country);
                }
            }
            for i in 0..words.len() {
                if false {
                    // words[i] = num;
                } else {
                    if let Some(num) = countries.get(words[i]) {
                        words[i] = num;
                    }
                }
            }
            let joined = words.join(" ");
            Some(joined)
        }
    }).collect();
    vec
}

fn parse(input: String) {
    let mut side = Side::US;
    let cards: HashMap<String, Card> = (1..Card::total()).map(|i| {
        let c = Card::from_index(i);
        let string = format!("{:?}", c);
        (string, c)
    }).collect();
    let vec: Vec<_> = input.lines().filter(|line| !line.starts_with("#")).collect();
}

fn side(x: &str) -> IResult<&str, &str> {
    nom::branch::alt((tag("USSR"), tag("US")))(x)
}

fn space(x: &str) -> IResult<&str, &str> {
    nom::bytes::complete::is_a(" \t")(x)
}

fn card(x: &str) -> IResult<&str, &str> {
    let word = nom::bytes::complete::is_not(" \t")(x);
    if let Ok((_, w)) = word {
        if let Some(_) = CARDS.get(w) {
            word // Valid card
        } else {
            Err(Error((x, ErrorKind::IsNot))) // Todo improve this
        }
    } else {
        Err(Error((x, ErrorKind::IsNot))) // Todo improve this
    }
}

// fn parse(x: &str) -> IResult<&str, &str> {
//     tuple((side, space, card))
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_parse() {
        assert_eq!(side("USSR Place 20 20"), Ok((" Place 20 20", "USSR")));
        assert_eq!(side("US Warsaw_Pact_Formed OE"), Ok((" Warsaw_Pact_Formed OE", "US")));
        assert!(side("IDK").is_err());
        assert_eq!(card("Warsaw_Pact_Formed OE"), Ok((" OE", "Warsaw_Pact_Formed")));
        assert!(card("NotNATO").is_err());
    }
    #[test]
    fn line_parse() {
        let input = "US Warsaw_Pact_Formed OE";
        let (_, (side, _, card)) = tuple((side, space, card))(input).unwrap();
        dbg!(side);
        dbg!(card);
    }
}