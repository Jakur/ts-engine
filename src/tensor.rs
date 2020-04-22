use crate::action::{Action, Decision};
use crate::card::{Card, Effect};
use crate::country::{CName, Side};
use crate::state::GameState;

lazy_static! {
    pub static ref SPECIAL_TOTAL: usize = {
        let last = CARD_OFFSET.last_value();
        let card = CARD_OFFSET.last_key();
        card.max_e_choices() + last
    };
    static ref CARD_OFFSET: IndexMap<Card, usize> = {
        let mut sum = 0;
        let x = (1..Card::total())
            .filter_map(|c| {
                let c = Card::from_index(c);
                let choices = c.max_e_choices();
                if choices <= 1 {
                    None
                } else {
                    let x = Some((c, sum));
                    sum += choices;
                    x
                }
            })
            .collect();
        IndexMap::new(x)
    };
}
pub struct DecodedChoice {
    pub action: Action,
    pub choice: Option<usize>,
}

impl DecodedChoice {
    pub fn new(action: Action, choice: Option<usize>) -> Self {
        Self { action, choice }
    }
}

impl std::fmt::Debug for DecodedChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use Action::*;
        if let Some(c) = self.choice {
            let c_str = match self.action {
                Ops | OpsEvent | Event | EventOps | Space | Discard => {
                    format!("{:?}", Card::from_index(c))
                }
                Influence | Coup | Realignment | Place | Remove | War => {
                    format!("{:?}", CName::from_index(c))
                }
                _ => format!("{:?}", c),
            };
            write!(f, "[{:?}: {:?}]", self.action, c_str)
        } else {
            write!(f, "[{:?}: {:?}]", self.action, self.choice)
        }
    }
}

pub type OutputVec = Vec<OutputIndex>;

fn encode_offsets(data: Vec<usize>) -> OutputVec {
    data.into_iter().map(|x| OutputIndex::new(x)).collect()
}

impl Decision {
    fn encode_begin_ar(&self, state: &GameState) -> OutputVec {
        let side = self.agent;
        // Quagmire / Bear Trap
        if (side == Side::US && state.has_effect(side, Effect::Quagmire))
            || (side == Side::USSR && state.has_effect(side, Effect::BearTrap))
        {
            let can_discard = state.cards_at_least(state.side, 2);
            if can_discard.is_empty() {
                // Must play scoring cards then pass
                let scoring = state.deck.scoring_cards(side);
                if scoring.is_empty() {
                    return encode_offsets(vec![Action::Pass.offset()]);
                } else {
                    let scoring: Vec<_> = scoring.into_iter().map(|c| c as usize).collect();
                    let mut d = Decision::new(side, Action::Event, scoring);
                    return d.encode(state);
                }
            } else {
                // If must play scoring cards for the rest of the turn
                if state
                    .deck
                    .must_play_scoring(side, state.max_ar(side) - state.ar)
                {
                    let scoring = state.deck.scoring_cards(side);
                    let scoring: Vec<_> = scoring.into_iter().map(|c| c as usize).collect();
                    let mut x = Decision::new(side, Action::Event, scoring);
                    return x.encode(state);
                }
                // Else discard normally
                let legal: Vec<_> = can_discard.into_iter().map(|x| x as usize).collect();
                let mut x = Decision::new(state.side, Action::Discard, legal);
                return x.encode(state);
            }
        }
        let space = state.legal_space(self.agent);
        let mut space_d = Decision::new(self.agent, Action::Space, space);
        let mut out = space_d.encode(state);
        let cc = |vec: Vec<Card>| vec.into_iter().map(|c| c as usize).collect::<Vec<_>>();
        let before_after = cc(state.deck.opp_events_fire(side, state));
        let event = cc(state.deck.can_event(side, state));
        let ops = cc(state.deck.can_play_ops(side, state));
        let mut e_ops = Decision::new(self.agent, Action::EventOps, before_after.clone());
        let mut ops_e = Decision::new(self.agent, Action::OpsEvent, before_after);
        let mut e = Decision::new(self.agent, Action::Event, event);
        let mut ops = Decision::new(self.agent, Action::Ops, ops);
        out.extend(e_ops.encode(state));
        out.extend(ops_e.encode(state));
        out.extend(e.encode(state));
        out.extend(ops.encode(state));
        // Todo rarer things like discarding with space power
        out
    }
}

pub(crate) trait TensorOutput {
    fn encode(&mut self, state: &GameState) -> OutputVec;
}

impl TensorOutput for Decision {
    fn encode(&mut self, state: &GameState) -> OutputVec {
        let begin = self.action.offset();
        let out = match self.action {
            Action::BeginAr => {
                let mut standard = if state.has_effect(self.agent, Effect::MissileEnvy) {
                    let mut d =
                        Decision::new(self.agent, Action::Ops, vec![Card::Missile_Envy as usize]);
                    d.encode(state)
                } else {
                    self.encode_begin_ar(state)
                };
                if state
                    .deck
                    .must_play_scoring(self.agent, state.ar_left(self.agent))
                {
                    // Check to see if we're already allowed to play a scoring card
                    let scoring_play = standard.iter().any(|x| {
                        let decoded = x.decode();
                        if decoded.action == Action::Event {
                            let card = decoded.choice.map(|i| Card::from_index(i)).unwrap();
                            card.is_scoring()
                        } else {
                            false
                        }
                    });
                    if !scoring_play {
                        // If not add scoring card events to our legal list
                        let scoring = state.deck.scoring_cards(self.agent);
                        standard.extend(scoring.into_iter().map(|card| {
                            let a = Action::Event;
                            let offset = card as usize;
                            OutputIndex::new(a.offset() + offset)
                        }));
                    }
                }
                standard
            }
            Action::ConductOps => {
                let inf = state.legal_influence(self.agent, self.quantity);
                let mut inf_d = Decision::new(self.agent, Action::Influence, inf);
                let mut out = inf_d.encode(state);
                // Todo coup restrictions
                let coup_realign = state.legal_coup_realign(self.agent);
                if !state.has_effect(self.agent, Effect::CubanMissileCrisis) {
                    let mut coup_d = Decision::new(self.agent, Action::Coup, coup_realign.clone());
                    out.extend(coup_d.encode(state));
                }
                let mut realign_d = Decision::new(self.agent, Action::Realignment, coup_realign);
                out.extend(realign_d.encode(state));
                out
            }
            Action::CubanMissile => {
                // This is somewhat clunky, but should work
                // Todo include pass or empty
                let pass = Action::Pass.offset();
                let mut out = encode_offsets(vec![pass]);
                let cuban_offset = self.action.offset();
                let remove = self
                    .allowed
                    .slice(state)
                    .iter()
                    .copied()
                    .map(|x| x + cuban_offset)
                    .collect();
                out.extend(encode_offsets(remove));
                out
            }
            Action::Coup if state.has_effect(self.agent, Effect::CubanMissileCrisis) => {
                // Todo include pass or empty?
                encode_offsets(vec![Action::Pass.offset()])
            }
            _ => {
                let v = self
                    .allowed
                    .slice(state)
                    .iter()
                    .copied()
                    .map(|x| x + begin)
                    .collect();
                encode_offsets(v)
            }
        };
        out
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct OutputIndex {
    data: usize,
}

impl OutputIndex {
    pub fn new(data: usize) -> OutputIndex {
        OutputIndex { data }
    }
    pub fn pass() -> OutputIndex {
        OutputIndex::new(Action::Pass.offset())
    }
    // Encode a single action-choice pair.
    pub fn encode_single(action: Action, choice: usize) -> Self {
        Self::new(action.offset() + choice)
    }
    pub fn decode(&self) -> DecodedChoice {
        let x = Action::action_from_offset(self.data);
        DecodedChoice::new(x.0, Some(x.1))
    }
    pub fn inner(&self) -> usize {
        self.data
    }
}

impl std::fmt::Debug for OutputIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let decoded = self.decode();
        write!(f, "{:?}", decoded)
    }
}

pub struct IndexMap<K, V> {
    keys: Vec<K>,
    values: Vec<Option<V>>,
}

impl<K: Into<usize> + Copy, V: std::cmp::Ord + Copy> IndexMap<K, V> {
    pub fn new(mut pairs: Vec<(K, V)>) -> Self {
        pairs.sort_by(|(_x1, y1), (_x2, y2)| y1.cmp(y2));
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (k, v) in pairs.into_iter() {
            keys.push(k);
            let k: usize = k.into();
            while values.len() <= k {
                values.push(None);
            }
            values[k] = Some(v);
        }
        IndexMap { keys, values }
    }
    pub fn get(&self, key: K) -> Option<&V> {
        let key = key.into();
        if key >= self.values.len() {
            None
        } else {
            self.values[key].as_ref()
        }
    }
    #[cfg(test)]
    pub fn find_key(&self, value: &V) -> K {
        // let index = self.values.binary_search(value)?;
        let index = self
            .keys
            .binary_search_by_key(value, |k| {
                let i: usize = (*k).into();
                let v = self.values[i];
                v.unwrap()
            })
            .unwrap();
        self.keys[index]
    }
    pub fn last_value(&self) -> &V {
        self.values.last().unwrap().as_ref().unwrap()
    }
    pub fn last_key(&self) -> &K {
        self.keys.last().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::country::Side;

    #[test]
    fn test_index_map() {
        let pairs: Vec<(usize, usize)> = vec![(5, 0), (10, 2), (25, 4), (30, 7), (64, 10)];
        let map = IndexMap::new(pairs.clone());
        for (x, y) in pairs.iter().copied() {
            assert_eq!(*map.get(x).unwrap(), y);
        }
        let mut ptr = 0;
        for i in 0..16usize {
            if let Some((_x, y)) = pairs.get(ptr + 1) {
                if i >= *y {
                    ptr += 1;
                }
            }
            let (x, y) = pairs[ptr];
            assert_eq!(map.find_key(&y), x);
        }
    }

    #[test]
    fn begin_ar() {
        let mut state = GameState::new();
        let mut hand: Vec<_> = (14..21).map(|x| Card::from_index(x)).collect();
        hand.push(Card::Asia_Scoring);
        state.deck.us_hand_mut().extend(hand);
        let side = Side::US;
        let d = Decision::new(side, Action::BeginAr, &[]);
        let output_vec = d.encode(&state);
        let mut events = 0;
        let mut spaces = 0;
        let out_vec_len = output_vec.len();
        for out in output_vec {
            let decoded = out.decode();
            match decoded.action {
                Action::Event => events += 1,
                Action::Space => spaces += 1,
                _ => {}
            }
        }
        assert_eq!(out_vec_len, 15 + 4);
        assert_eq!(events, 4);
        assert_eq!(spaces, 4);
    }

    #[test]
    fn conduct_ops() {
        use crate::country::CName::*;
        use std::collections::HashSet;
        let state = GameState::four_four_two();
        let d = Decision::new(Side::USSR, Action::ConductOps, &[]);
        let output_vec = d.encode(&state);
        let mut coup = 0;
        let mut realign = 0;
        let mut inf: HashSet<_> = vec![
            Finland,
            Sweden,
            EGermany,
            Romania,
            Poland,
            Czechoslovakia,
            Austria,
            Hungary,
            Turkey,
            Syria,
            Lebanon,
            Israel,
            Iraq,
            Jordan,
            SaudiaArabia,
            GulfStates,
            Afghanistan,
            NKorea,
            SKorea,
        ]
        .into_iter()
        .map(|x| x as usize)
        .collect();
        for out in output_vec {
            let decoded = out.decode();
            match decoded.action {
                Action::Coup => coup += 1,
                Action::Realignment => realign += 1,
                Action::Influence => {
                    let in_set = inf.remove(&decoded.choice.unwrap());
                    assert!(in_set);
                }
                _ => {}
            }
        }
        assert_eq!(coup, 12);
        assert_eq!(realign, 12);

        assert_eq!(inf.len(), 0);
    }
}
