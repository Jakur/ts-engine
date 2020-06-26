use crate::card::{Card, Effect};
use crate::country::Side;
use crate::tensor::DecodedChoice;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::collections::HashMap;

lazy_static! {
    static ref CARD_NAMES: HashMap<String, Card> = {
        let mut map: HashMap<_, _> = (1..Card::total())
            .map(|num| {
                let card = Card::from_index(num);
                (standard_card_name(card), card)
            })
            .collect();
        let fix = [("John Paul II Elected Pope*", Card::John_Paul)];
        for (k, v) in fix.iter() {
            map.insert(k.to_string(), *v);
        }
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

// fn get_card(name: &str) -> Option<Card> {
//     CARD_NAMES.get(name.split_whitespace().next()?).copied()
// }

fn get_country(name: &str) -> Option<usize> {
    COUNTRY_INDEX.get(name.split_whitespace().next()?).copied()
}

fn parse_num<T: std::str::FromStr>(num: Pair<Rule>) -> Option<T> {
    num.as_str().parse::<T>().ok()
}

#[derive(Debug)]
struct CountryChange {
    index: usize,
    us: i8,
    ussr: i8,
    delta: i8,
}

impl CountryChange {
    fn new(index: usize, us: i8, ussr: i8, delta: i8) -> Self {
        Self {
            index,
            us,
            ussr,
            delta,
        }
    }
}

#[derive(Debug)]
struct MilOps {
    side: Side,
    ops: i8,
}

impl MilOps {
    fn new(side: Side, ops: i8) -> Self {
        Self { side, ops }
    }
}

#[derive(Debug)]
// Outcome Order: US, USSR
enum Outcome {
    Country(CountryChange),
    Defcon(i8),
    Vp(i8),
    StartEffect(Effect), // Todo side of effect? (Harder than one would think)
    EndEffect(Effect),
    MilitaryOps(MilOps),
    War {
        target: usize,
        roll: i8,
        changes: Vec<CountryChange>,
        mil_ops: MilOps,
    },
    Space(Side, i8),
}

fn parse_outcome(pair: Pair<Rule>) -> Option<Outcome> {
    let rule = pair.as_rule();
    let mut vals = pair.into_inner().into_iter();
    match rule {
        Rule::outcome => {
            // Convert to its inner type and recurse
            let inner = vals.next().expect("Must have an inner value");
            parse_outcome(inner)
        }
        Rule::country_change => {
            let delta = parse_num(vals.nth(1)?)?;
            let name = vals.next().unwrap().as_str();
            let index = get_country(name)?;
            let (us, ussr) = parse_country_status(vals.next().unwrap())?;
            Some(Outcome::Country(CountryChange {
                index,
                us,
                ussr,
                delta,
            }))
        }
        Rule::vp_change => {
            // Todo see if this actually works
            let vp = if let Some(s) = vals.next() {
                let side = parse_side(s)?;
                let num = parse_num(vals.next()?)?;
                match side {
                    Side::US => num,
                    Side::USSR => num * -1,
                    Side::Neutral => unimplemented!(),
                }
            } else {
                0
            };
            Some(Outcome::Vp(vp))
        }
        Rule::start_effect => {
            let card = parse_card(vals.next()?)?;
            let effect = Effect::card_to_effect(card)?;
            Some(Outcome::StartEffect(effect))
        }
        Rule::end_effect => {
            let card = parse_card(vals.next()?)?;
            let effect = Effect::card_to_effect(card)?;
            Some(Outcome::EndEffect(effect))
        }
        Rule::set_mil_ops => {
            let side = parse_side(vals.next()?)?;
            let num = parse_num(vals.next()?)?;
            Some(Outcome::MilitaryOps(MilOps::new(side, num)))
        }
        Rule::war => {
            let name = vals.next()?.as_str();
            let target = get_country(name)?;
            let roll = parse_num(vals.next()?)?;
            let mut changes = Vec::new();
            while let Some(pair) = vals.next() {
                // Add country changes until we hit mil ops, i.e. the last line
                let outcome = parse_outcome(pair)?;
                match outcome {
                    Outcome::Country(cc) => {
                        changes.push(cc);
                    }
                    Outcome::MilitaryOps(mil_ops) => {
                        return Some(Outcome::War {
                            target,
                            roll,
                            changes,
                            mil_ops,
                        })
                    }
                    _ => return None,
                }
            }
            None
        }
        Rule::space => {
            let side = parse_side(vals.next()?)?;
            let level = vals.next()?.as_str().parse().ok()?;
            Some(Outcome::Space(side, level))
        }
        _ => None,
    }
}

fn parse_country_status(pair: Pair<Rule>) -> Option<(i8, i8)> {
    match pair.as_rule() {
        Rule::country_status => {
            let mut vals = pair.into_inner();
            let us = vals.next().unwrap().as_str().parse().ok()?;
            let ussr = vals.next().unwrap().as_str().parse().ok()?;
            Some((us, ussr))
        }
        _ => None,
    }
}

enum AR<'a> {
    EventBefore {
        event: Pair<'a, Rule>,
        action: Pair<'a, Rule>,
        card: Card,
    },
    EventAfter {
        action: Pair<'a, Rule>,
        event: Pair<'a, Rule>,
        card: Card,
    },
    SimpleUse {
        action: Pair<'a, Rule>,
        card: Card,
    },
}

impl<'a> AR<'a> {
    fn consume(self, primary_side: Side) -> Vec<ActionRound> {
        match self {
            AR::EventBefore {
                event,
                action,
                card,
            } => {
                let e_side = card.side();
            }
            _ => {}
        }
        vec![]
    }
}

fn parse_action(action: Pair<Rule>) -> ActionRound {
    use std::cell::RefCell;
    let choices = RefCell::new(Vec::new());
    let outcomes = RefCell::new(Vec::new());
    inner(action, &choices, &outcomes);
    fn inner(
        x: Pair<Rule>,
        choices: &RefCell<Vec<DecodedChoice>>,
        outcomes: &RefCell<Vec<Outcome>>,
    ) {
        match x.as_rule() {
            Rule::event => todo!(),
            Rule::inf => {}
            _ => todo!(),
        }
    }
    let choices = choices.into_inner();
    let outcomes = outcomes.into_inner();
    ActionRound {
        choices,
        outcomes,
        card: None,
        actor: Side::Neutral,
    }
}

#[derive(Debug)]
pub struct ActionRound {
    choices: Vec<DecodedChoice>,
    outcomes: Vec<Outcome>,
    card: Option<Card>,
    actor: Side,
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
                card: Some(c),
                actor: Side::Neutral,
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
            // The key is the first word, i.e. until the next uppercase char
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
            if count == 0 {
                let ar = parse_ar(parsed);
                assert_eq!(
                    Some(Card::John_Paul),
                    ar.map(|x| x.card.expect("Some card"))
                );
            }
            // dbg!(parsed);
        }
    }
}
