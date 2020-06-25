use crate::card::{Card, Effect};
use crate::country::Side;
use crate::tensor::DecodedChoice;
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

lazy_static! {
    static ref CARD_NAMES: HashMap<String, Card> = {
        let map = (1..Card::total())
            .map(|num| {
                let card = Card::from_index(num);
                (standard_card_name(card), card)
            })
            .collect();
        map
    };
    static ref COUNTRY_INDEX: HashMap<String, usize> = { country_names() };
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TwilightParser;

fn standard_card_name(card: Card) -> String {
    let star = if card.is_starred() { "*" } else { "" };
    let name = format!("{:?}{}", card, star).replace("_", " ");
    name
}

#[derive(Debug)]
// Outcome Order: US, USSR
enum Outcome {
    Country { index: usize, us: i8, ussr: i8 },
    Defcon(i8),
    Vp(i8),
    StartEffect(Effect), // Todo side of effect? (Harder than one would think)
    EndEffect(Effect),
    MilOps(Side, i8),
    War, // Todo
    Space(Side, i8),
    ConductOps, // Todo
}

fn parse_outcome(pair: Pair<Rule>) -> Option<Outcome> {
    unimplemented!();
}

#[derive(Debug)]
pub struct ActionRound {
    choices: Vec<DecodedChoice>,
    outcomes: Vec<Outcome>,
    card: Card, // Debug only
}

fn parse_ar(pair: Pair<Rule>) -> Option<ActionRound> {
    // dbg!(pair.as_rule());
    let pair = pair.into_inner().peek().unwrap();
    // dbg!(pair.as_rule());
    match pair.as_rule() {
        Rule::turn_std => {
            let children: Vec<_> = pair.into_inner().collect();
            // dbg!(&children);
            for c in children.iter() {
                eprintln!("{:?}", c.as_rule());
            }
            let card = children
                .into_iter()
                .map(|x| parse_card(x))
                .find(|x| x.is_some())
                .map(|x| x.unwrap());
            card.map(|c| ActionRound {
                choices: vec![],
                outcomes: vec![],
                card: c,
            })
        }
        _ => None,
    }
}

fn parse_card(pair: Pair<Rule>) -> Option<Card> {
    match pair.as_rule() {
        Rule::card => Some(
            *CARD_NAMES
                .get(pair.as_str())
                .expect(&format!("{} is valid", pair.as_str())),
        ),
        _ => None,
    }
}

fn parse_side(pair: Pair<Rule>) -> Option<Side> {
    match pair.as_rule() {
        Rule::side => match pair.as_str() {
            "US" => Some(Side::US),
            "USSR" => Some(Side::USSR),
            _ => None,
        },
        _ => None,
    }
}

fn country_names() -> HashMap<String, usize> {
    use crate::country::CName;
    let names: Vec<_> = (0..CName::total() - 2)
        .map(|x| CName::from_index(x))
        .collect();
    let mut first: HashMap<_, _> = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let s = format!("{:?}", name);
            let end = s
                .char_indices()
                .skip(1)
                .find(|(_i, c)| c.is_ascii_uppercase());
            if let Some((pos, _)) = end {
                (format!("{}", &s[0..pos]), index)
            } else {
                (s, index)
            }
        })
        .collect();
    let fix = [
        ("North", CName::NKorea),
        ("South", CName::SKorea),
        ("SE", CName::SEAfricanStates),
        ("UK", CName::UK),
        ("West", CName::WGermany),
        ("East", CName::EGermany),
    ];
    for (k, v) in fix.iter() {
        first.insert(k.to_string(), *v as usize);
    }
    first
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn card_names() {
        for i in 1..Card::total() {
            let card = Card::from_index(i);
            eprintln!("{}", standard_card_name(card));
        }
    }
    #[test]
    fn test() {
        let f = "Turn 6, USSR AR6
John Paul II Elected Pope*
Event: John Paul II Elected Pope*
USSR -2 in Poland [0][4]
US +1 in Poland [1][4]
John Paul II Elected Pope* is now in play.

Place Influence (2 Ops): 
USSR +1 in Venezuela [0][2]
USSR +1 in South Africa [3][2]
";
        let f2 = "Turn 6, US AR3
Portuguese Empire Crumbles*
Event: Portuguese Empire Crumbles*
USSR +2 in SE African States [0][2]
USSR +2 in Angola [4][4]

Realignment (2 Ops): 
Target: Angola
USSR rolls 6
US rolls 4 (+2) = 6
Target: Angola
USSR rolls 1
US rolls 5 (+2) = 7
USSR -4 in Angola [4][0]
";

        let f3 = "Turn 6, USSR AR2
Che
Event: Che
Coup (3 Ops):
Target: Colombia
SUCCESS: 1 [ + 3 - 2x1 = 2 ]
US -1 in Colombia [0][0]
USSR +1 in Colombia [0][1]
USSR Military Ops to 5

Coup (3 Ops):
Target: Cameroon
SUCCESS: 3 [ + 3 - 2x1 = 4 ]
US -1 in Cameroon [0][0]
USSR +3 in Cameroon [0][3]
USSR Military Ops to 5
        
";

        let f4 = "Turn 7, USSR AR1
\"One Small Step...\"
Coup (2 Ops): 
Target: Nigeria
SUCCESS: 3 [ + 2 - 2x1 = 3 ]
US -1 in Nigeria [0][0]
USSR +2 in Nigeria [0][2]
USSR Military Ops to 2
DEFCON degrades to 2
        
";
        for (count, string) in [f, f2, f3, f4].iter().enumerate() {
            eprintln!("Attempting f{}", count + 1);
            let parsed = TwilightParser::parse(Rule::turn, &string)
                .expect("Bad parse")
                .next()
                .unwrap();
            if count == 1 {
                let ar = parse_ar(parsed);
                assert_eq!(Some(Card::Portuguese_Empire_Crumbles), ar.map(|x| x.card));
            }
            // dbg!(parsed);
        }
    }
}
