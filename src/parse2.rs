use crate::action::Action;
use crate::card::{Card, Effect};
use crate::country::Side;
use crate::tensor::DecodedChoice;
use anyhow::{anyhow, ensure, Result};
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
        let fix = [
            ("John Paul II Elected Pope*", Card::John_Paul),
            ("\"One Small Step...\"", Card::One_Small_Step),
        ];
        for (k, v) in fix.iter() {
            map.insert(k.to_string(), *v);
        }
        map
    };
    static ref COUNTRY_INDEX: HashMap<String, usize> = { country_names() };
}

#[derive(Debug)]
enum Change {
    Choice(DecodedChoice),
    Roll(Side, i8),
}

impl Change {
    fn choice(action: Action, val: Option<usize>) -> Self {
        Change::Choice(DecodedChoice::new(action, val))
    }
    fn roll(side: Side, val: i8) -> Self {
        Change::Roll(side, val)
    }
}

pub struct Replay {
    vec: Vec<FatPly>,
}

impl Replay {
    fn get_rolls(&self) -> (Vec<i8>, Vec<i8>) {
        let mut us_rolls = Vec::new();
        let mut ussr_rolls = Vec::new();
        for fp in self.vec.iter() {
            for ch in fp.choices.iter().rev() {
                if let Change::Roll(mut side, roll) = ch {
                    // Neutral represents the initiating side
                    if let Side::Neutral = side {
                        side = fp.actor;
                    }
                    match side {
                        Side::US => us_rolls.push(*roll),
                        Side::USSR => ussr_rolls.push(*roll),
                        _ => unreachable!(),
                    }
                }
            }
        }
        (us_rolls, ussr_rolls)
    }
}

#[derive(Debug)]
struct FatPly {
    choices: Vec<Change>,
    outcomes: Vec<Outcome>,
    card: Option<Card>,
    actor: Side,
}

impl FatPly {
    /// Moves all the choices and outcomes of other into Self, leaving other empty,
    /// mimicking the behavior of Vec::append().
    fn append(&mut self, other: &mut FatPly) {
        self.choices.append(&mut other.choices);
        self.outcomes.append(&mut other.outcomes);
    }
}

pub fn parse_game() -> Replay {
    todo!()
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TwilightParser;

fn standard_card_name(card: Card) -> String {
    let star = if card.is_starred() { "*" } else { "" };
    let name = format!("{:?}{}", card, star).replace("_", " ");
    name
}

fn get_country(name: &str) -> Result<usize> {
    let search = if name == "South Africa" {
        name
    } else {
        name.split_whitespace().next().unwrap()
    };
    COUNTRY_INDEX
        .get(search)
        .copied()
        .ok_or_else(|| anyhow!("Cannot find country: {}", search))
}

fn parse_num<T: std::str::FromStr>(num: Pair<Rule>) -> Result<T> {
    num.as_str()
        .parse::<T>()
        .map_err(|_| anyhow!("Failed num parse"))
}

#[derive(Debug, PartialEq)]
struct CountryChange {
    side: Side,
    index: usize,
    us: i8,
    ussr: i8,
    delta: i8,
}

impl CountryChange {
    fn new(side: Side, index: usize, us: i8, ussr: i8, delta: i8) -> Self {
        Self {
            side,
            index,
            us,
            ussr,
            delta,
        }
    }
}

#[derive(Debug, PartialEq)]
struct MilOps {
    side: Side,
    ops: i8,
}

impl MilOps {
    fn new(side: Side, ops: i8) -> Self {
        Self { side, ops }
    }
}

#[derive(Debug, PartialEq)]
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

fn parse_outcome(pair: Pair<Rule>) -> Result<Outcome> {
    let rule = pair.as_rule();
    let mut vals = pair.into_inner().into_iter();
    match rule {
        Rule::outcome => {
            // Convert to its inner type and recurse
            let inner = vals.next().expect("Must have an inner value");
            parse_outcome(inner)
        }
        Rule::country_change => {
            let side = parse_side(vals.next().unwrap())?;
            let delta = parse_num(vals.next().unwrap())?;
            let name = vals.next().unwrap().as_str();
            let index = get_country(name)?;
            let (us, ussr) = parse_country_status(vals.next().unwrap())?;
            Ok(Outcome::Country(CountryChange {
                side,
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
                let num = parse_num(vals.next().unwrap())?;
                match side {
                    Side::US => num,
                    Side::USSR => num * -1,
                    Side::Neutral => unimplemented!(),
                }
            } else {
                0
            };
            Ok(Outcome::Vp(vp))
        }
        Rule::start_effect => {
            let card = parse_card(vals.next().unwrap())?;
            let effect = Effect::card_to_effect(card)
                .ok_or_else(|| anyhow!("No effect found for card: {:?}", card))?;
            Ok(Outcome::StartEffect(effect))
        }
        Rule::end_effect => {
            let card = parse_card(vals.next().unwrap())?;
            let effect = Effect::card_to_effect(card)
                .ok_or_else(|| anyhow!("No effect found for card: {:?}", card))?;
            Ok(Outcome::EndEffect(effect))
        }
        Rule::set_mil_ops => {
            let side = parse_side(vals.next().unwrap())?;
            let num = parse_num(vals.next().unwrap())?;
            Ok(Outcome::MilitaryOps(MilOps::new(side, num)))
        }
        Rule::war => {
            let name = vals.next().unwrap().as_str();
            let target = get_country(name)?;
            let roll = parse_num(vals.next().unwrap())?;
            let mut changes = Vec::new();
            while let Some(pair) = vals.next() {
                // Add country changes until we hit mil ops, i.e. the last line
                let outcome = parse_outcome(pair)?;
                match outcome {
                    Outcome::Country(cc) => {
                        changes.push(cc);
                    }
                    Outcome::MilitaryOps(mil_ops) => {
                        return Ok(Outcome::War {
                            target,
                            roll,
                            changes,
                            mil_ops,
                        })
                    }
                    _ => return Err(anyhow!("Invalid outcome for war: {:?}", outcome)),
                }
            }
            return Err(anyhow!("Never parsed war military ops!"));
        }
        Rule::space => {
            let side = parse_side(vals.next().unwrap())?;
            let level = vals.next().unwrap().as_str().parse()?;
            Ok(Outcome::Space(side, level))
        }
        Rule::defcon_change => {
            let defcon = parse_num(vals.next().unwrap())?;
            Ok(Outcome::Defcon(defcon))
        }
        _ => Err(anyhow!("Invalid outcome rule: {:?}", rule)),
    }
}

fn parse_country_status(pair: Pair<Rule>) -> Result<(i8, i8)> {
    ensure!(pair.as_rule() == Rule::country_status, "Wrong Rule");
    let mut vals = pair.into_inner();
    let us = vals.next().unwrap().as_str().parse()?;
    let ussr = vals.next().unwrap().as_str().parse()?;
    Ok((us, ussr))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ARInfo {
    turn: i8,
    ar: i8,
    side: Side,
    card: Card,
}

impl ARInfo {
    fn new(turn: i8, ar: i8, side: Side, card: Card) -> Self {
        Self {
            turn,
            ar,
            side,
            card,
        }
    }
}

fn parse_action(action: Pair<Rule>, mut actor: Side, card: Option<Card>) -> FatPly {
    use std::cell::RefCell;
    let choices = RefCell::new(Vec::new());
    let outcomes = RefCell::new(Vec::new());
    let res = inner(action, &choices, &outcomes, card);
    match res {
        Ok(side) => match side {
            Side::USSR | Side::US => actor = side,
            _ => {}
        },
        Err(e) => {
            eprintln!("{}", e.root_cause());
            panic!("Bad parse!");
        }
    }
    fn inner(
        x: Pair<Rule>,
        choices: &RefCell<Vec<Change>>,
        outcomes: &RefCell<Vec<Outcome>>,
        card: Option<Card>,
    ) -> Result<Side> {
        let mut side = Side::Neutral;
        match x.as_rule() {
            Rule::card_use => {
                let child = x.into_inner().into_iter().next().unwrap();
                return inner(child, choices, outcomes, card);
            }
            Rule::event => {
                let mut iter = x.into_inner().into_iter();
                let c = parse_card(iter.next().unwrap())?;
                if c != card.expect("Valid card") {
                    todo!() // Events calling other events!
                }
                while let Some(pair) = iter.next() {
                    match pair.as_rule() {
                        Rule::conduct_ops => {
                            let child = pair.into_inner().into_iter().next().unwrap();
                            side = inner(child, choices, outcomes, card)?;
                        }
                        Rule::outcome => {
                            let outcome = parse_outcome(pair)?;
                            outcomes.borrow_mut().push(outcome);
                        }
                        _ => unimplemented!(),
                    }
                }
                // Assume for now card event side has priority over outcome side
                match c.side() {
                    Side::USSR | Side::US => side = c.side(),
                    Side::Neutral => {}
                }
            }
            Rule::inf => {
                let mut iter = x.into_inner().into_iter().skip(1);
                while let Some(pair) = iter.next() {
                    let out = parse_outcome(pair)?;
                    if let Outcome::Country(ref cc) = out {
                        side = cc.side; // Implies Side is placing inf
                        let index = cc.index;
                        let choice = DecodedChoice::new(Action::Influence, Some(index));
                        choices.borrow_mut().push(Change::Choice(choice));
                        outcomes.borrow_mut().push(out);
                    } else {
                        panic!("Expected country change!");
                    }
                }
            }
            Rule::coup => {
                let mut iter = x.into_inner().into_iter().skip(1);
                let target = parse_target(iter.next().unwrap())?;
                let roll = iter.next().unwrap().into_inner().peek().unwrap();
                let roll = roll.as_str().parse().expect("Valid number");
                let mut outs = outcomes.borrow_mut();
                while let Some(pair) = iter.next() {
                    let out = parse_outcome(pair)?;
                    // Try to determine who is couping. This should only fail on
                    // a failed free coup.
                    match out {
                        Outcome::Country(ref cc) => {
                            if cc.delta > 0 {
                                side = cc.side;
                            } else {
                                side = cc.side.opposite();
                            }
                        }
                        Outcome::MilitaryOps(ref mil_ops) => {
                            side = mil_ops.side;
                        }
                        _ => {}
                    }
                    outs.push(out);
                }
                let mut chs = choices.borrow_mut();
                chs.push(Change::choice(Action::Coup, Some(target)));
                chs.push(Change::roll(Side::Neutral, roll));
            }
            Rule::realign => {
                let mut iter = x.into_inner().into_iter().skip(1);
                let mut chs = choices.borrow_mut();
                let mut outs = outcomes.borrow_mut();
                while let Some(attempt) = iter.next() {
                    let mut iter = attempt.into_inner().into_iter();
                    let target = parse_target(iter.next().unwrap())?;
                    let roll = parse_roll(iter.next().unwrap())?;
                    let roll2 = parse_roll(iter.next().unwrap())?;
                    if let Some(outcome) = iter.next() {
                        let outcome = parse_outcome(outcome)?;
                        outs.push(outcome);
                    }
                    chs.push(Change::choice(Action::Realignment, Some(target)));
                    chs.push(roll);
                    chs.push(roll2);
                }
            }
            _ => {
                dbg!(x.as_rule());
                todo!();
            }
        }
        Ok(side)
    }
    // Remove the Refcell layer
    let choices = choices.into_inner();
    let outcomes = outcomes.into_inner();
    FatPly {
        choices,
        outcomes,
        card,
        actor,
    }
}

fn parse_ar(pair: Pair<Rule>) -> Result<AR> {
    let pair = pair.into_inner().into_iter().next().unwrap(); // Debug misdirection
    let pair = pair.into_inner().peek().unwrap();
    match pair.as_rule() {
        Rule::turn_std => {
            let mut children = pair.into_inner().into_iter();
            let turn: i8 = parse_num(children.next().unwrap())?;
            let side = parse_side(children.next().unwrap())?;
            let ar: i8 = parse_num(children.next().unwrap())?;

            if let Some(card) = children.next() {
                let card = parse_card(card)?;
                let info = ARInfo::new(turn, ar, side, card);
                let ar = AR::try_new(children.next().expect("Valid"), info);
                ar
            } else {
                todo!() // Pass action
            }
        }
        _ => todo!(),
    }
}

#[derive(Debug)]
struct AR {
    actions: Vec<FatPly>,
    info: ARInfo,
    timing: Timing,
}

impl AR {
    fn try_new(pair: Pair<Rule>, info: ARInfo) -> Result<Self> {
        ensure!(pair.as_rule() == Rule::play_card, "Wrong rule!");
        let mut actions = Vec::new();
        let mut iter = pair.into_inner().into_iter();
        while let Some(card_use) = iter.next() {
            // Todo more nuanced side / card
            let action = parse_action(card_use, info.side, Some(info.card));
            actions.push(action);
        }
        Ok(Self {
            actions,
            info,
            timing: Timing::Ops, // Todo
        })
    }
}

#[derive(Debug)]
enum Timing {
    EventBefore,
    EventAfter,
    Ops,
    Event,
}

fn parse_roll(pair: Pair<Rule>) -> Result<Change> {
    ensure!(pair.as_rule() == Rule::rolls, "Wrong Rule");
    let mut inner = pair.into_inner().into_iter();
    let side = parse_side(inner.next().unwrap())?;
    let roll = parse_num(inner.next().unwrap())?;
    Ok(Change::roll(side, roll))
}

fn parse_target(pair: Pair<Rule>) -> Result<usize> {
    let name = pair.into_inner().peek().unwrap().as_str();
    get_country(name)
}

fn parse_card(pair: Pair<Rule>) -> Result<Card> {
    ensure!(pair.as_rule() == Rule::card, "Wrong Rule");
    let name = pair.as_str();
    CARD_NAMES
        .get(name)
        .copied()
        .ok_or_else(|| anyhow!("No card found with name: {}", name))
}

fn parse_side(pair: Pair<Rule>) -> Result<Side> {
    ensure!(pair.as_rule() == Rule::side, "Wrong Rule");
    match pair.as_str() {
        "US" => Ok(Side::US),
        "USSR" => Ok(Side::USSR),
        _ => Err(anyhow!("Expected side got: {}", pair.as_str())),
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
        ("South Africa", CName::SouthAfrica),
    ];
    for (k, v) in fix.iter() {
        first.insert(k.to_string(), *v as usize);
    }
    first
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::country::CName;
    #[test]
    fn card_names() {
        for i in 1..Card::total() {
            let card = Card::from_index(i);
            eprintln!("{}", standard_card_name(card));
        }
    }
    #[test]
    fn test_file() {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::open("tests/1160218.txt").expect("Cannot open file");
        let mut string = String::new();
        file.read_to_string(&mut string).expect("Bad read");
        let _parsed = TwilightParser::parse(Rule::file, &string)
            .expect("Bad parse")
            .next()
            .unwrap();
    }
    #[test]
    fn test_hl() {
        let f = "Turn 1, Headline Phase
De Gaulle Leads France* & Mideast Scoring
USSR Headlines De Gaulle Leads France*
US Headlines Mideast Scoring
Event: De Gaulle Leads France*
USSR +1 in France [0][1]
De Gaulle Leads France* is now in play.

Event: Mideast Scoring
US gains 4 VP. Score is US 4.

";
        let _parsed = TwilightParser::parse(Rule::single_turn, &f)
            .expect("Bad parse")
            .next()
            .unwrap();
    }
    #[test]
    fn test_ars() {
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
            let parsed = TwilightParser::parse(Rule::single_turn, &string)
                .expect("Bad parse")
                .next()
                .unwrap();
            match count {
                0 => {
                    let ar = parse_ar(parsed).expect("Valid");
                    assert_eq!(ar.actions.len(), 2);
                    let event = &ar.actions[0];
                    let action = &ar.actions[1];
                    assert_eq!(event.actor, Side::US);
                    assert_eq!(action.actor, Side::USSR);
                    let event_outcomes = vec![
                        cc(event.actor, CName::Poland, 0, 4, -2),
                        cc(event.actor, CName::Poland, 1, 4, 1),
                        Outcome::StartEffect(Effect::AllowSolidarity),
                    ];
                    assert_eq!(event_outcomes, event.outcomes);
                    let action_outcomes = vec![
                        cc(action.actor, CName::Venezuela, 0, 2, 1),
                        cc(action.actor, CName::SouthAfrica, 3, 2, 1),
                    ];
                    assert_eq!(action_outcomes, action.outcomes);
                }
                1 => {
                    let ar = parse_ar(parsed).expect("Valid");
                    assert_eq!(ar.actions.len(), 2);
                    let event = &ar.actions[0];
                    let action = &ar.actions[1];
                    assert_eq!(event.actor, Side::USSR);
                    assert_eq!(action.actor, Side::US);
                    let event_outcomes = vec![
                        cc(event.actor, CName::SEAfricanStates, 0, 2, 2),
                        cc(event.actor, CName::Angola, 4, 4, 2),
                    ];
                    assert_eq!(event_outcomes, event.outcomes);
                    let action_outcomes = vec![cc(action.actor, CName::Angola, 4, 0, -4)];
                    assert_eq!(action_outcomes, action.outcomes);
                }
                2 => {
                    let ar = parse_ar(parsed).expect("Valid");
                    // Todo nicer way to test Che
                    let mut iter = ar.actions.into_iter();
                    let mut event = iter.next().unwrap();
                    while let Some(mut next) = iter.next() {
                        event.append(&mut next);
                    }
                    assert_eq!(event.actor, Side::USSR);
                    let event_outcomes = vec![
                        cc(event.actor, CName::Colombia, 0, 0, -1),
                        cc(event.actor, CName::Colombia, 0, 1, 1),
                        Outcome::MilitaryOps(MilOps::new(Side::USSR, 5)),
                        cc(event.actor, CName::Cameroon, 0, 0, -1),
                        cc(event.actor, CName::Cameroon, 0, 3, 3),
                        Outcome::MilitaryOps(MilOps::new(Side::USSR, 5)),
                    ];
                    assert_eq!(event_outcomes, event.outcomes);
                }
                3 => {
                    let ar = parse_ar(parsed).expect("Valid");
                    assert_eq!(ar.actions.len(), 1);
                    let action = &ar.actions[0];
                    assert_eq!(action.actor, Side::USSR);
                    let action_outcomes = vec![
                        cc(action.actor, CName::Nigeria, 0, 0, -1),
                        cc(action.actor, CName::Nigeria, 0, 2, 2),
                        Outcome::MilitaryOps(MilOps::new(Side::USSR, 2)),
                        Outcome::Defcon(2),
                    ];
                    assert_eq!(action_outcomes, action.outcomes);
                }
                _ => {}
            }
        }
    }
    fn cc(side: Side, cname: CName, us: i8, ussr: i8, delta: i8) -> Outcome {
        let side = if delta > 0 { side } else { side.opposite() };
        Outcome::Country(CountryChange::new(side, cname as usize, us, ussr, delta))
    }
}
