use crate::action::{Action, Decision, play_card_indices};
use crate::card::{self, Card, Effect};
use crate::country::Side;
use crate::state::GameState;

lazy_static! {
    pub static ref SPECIAL_TOTAL: usize = {
        let last = CARD_OFFSET.last_value();
        let card = CARD_OFFSET.last_key();
        card.max_e_choices() + last
    };
    static ref CARD_OFFSET: IndexMap<Card, usize> = {
        let mut sum = 0;
        let x = (1..card::NUM_CARDS + 1).filter_map(|c| {
            let c = Card::from_index(c);
            let choices = c.max_e_choices();
            if choices <= 1 {
                None
            } else {
                let x = Some((c, sum));
                sum += choices;
                x
            }
        }).collect();
        IndexMap::new(x)
    };

}

pub struct OutputVec {
    data: Vec<OutputIndex>
}

impl OutputVec {
    pub fn new(data: Vec<usize>) -> OutputVec {
        OutputVec{data: data.into_iter().map(|x| OutputIndex::new(x)).collect()}
    }
    pub fn data(&self) -> &Vec<OutputIndex> {
        &self.data
    }
    pub fn extend(&mut self, new_data: OutputVec) {
        for x in new_data.data.into_iter() {
            self.data.push(x);
        }
    }
    pub fn contains(&self, value: OutputIndex) -> bool {
        self.data.iter().find(|x| **x == value).is_some()
    }
}

pub(crate) trait TensorOutput {
    fn encode(&self, state: &GameState) -> OutputVec;
}

impl TensorOutput for Decision {
    fn encode(&self, state: &GameState) -> OutputVec {
        let begin = self.action.offset();
        let out = match self.action {
            Action::SpecialEvent => {
                let legal = state.legal_special_event(self.agent);
                let mut vec = Vec::new();
                for card in legal {
                    let choices = card.e_choices(state);
                    let card_offset = CARD_OFFSET.get(card).unwrap();
                    if let Some(v) = choices {
                        vec.extend(v.into_iter().map(|x| x + card_offset));
                    }
                } 
                OutputVec::new(vec)
            }
            Action::BeginAr => {
                let side = state.side;
                // Quagmire / Bear Trap
                if (side == Side::US && state.has_effect(side, Effect::Quagmire)) ||
                    (side == Side::USSR && state.has_effect(side, Effect::BearTrap)) {
                    let can_discard = state.cards_at_least(state.side, 2);
                    if can_discard.is_empty() {
                        // Must play scoring cards then pass
                        let scoring = state.deck.scoring_cards(side);
                        if scoring.is_empty() {
                            return OutputVec::new(vec![Action::Pass.offset()])
                        } else {
                            let scoring: Vec<_> = scoring.into_iter().map(|c| c as usize).collect();
                            let d = Decision::new(side, Action::Event, scoring);
                            return d.encode(state)
                        }
                    } else {
                        // If must play scoring cards for the rest of the turn
                        if state.deck.must_play_scoring(side, state.max_ar(side) - state.ar) {
                            let scoring = state.deck.scoring_cards(side);
                            let scoring: Vec<_> = scoring.into_iter().map(|c| c as usize).collect();
                            let x = Decision::new(side, Action::Event, scoring);
                            return x.encode(state)
                        }
                        // Else discard normally
                        let legal: Vec<_> = can_discard.into_iter().map(|x| x as usize).collect();
                        let x = Decision::new(state.side, Action::Discard, legal);
                        return x.encode(state)
                    }
                } 
                let space = state.legal_space(self.agent);
                let space_d = Decision::new(self.agent, Action::Space, space);
                let mut out = space_d.encode(state);
                let play_d = Decision::new(self.agent, Action::PlayCard, &[]);
                out.extend(play_d.encode(state));
                // Todo rarer things like discarding with space power
                out
            },
            Action::PlayCard => {
                if self.allowed.slice().len() == 0 {
                    play_card_indices(self.agent, state)
                } else {
                    OutputVec::new(self.allowed.slice().iter().copied().collect())
                }
            },
            Action::ConductOps => {
                let inf = state.legal_influence(self.agent, self.quantity);
                let inf_d = Decision::new(self.agent, Action::StandardOps, inf);
                let mut out = inf_d.encode(state);
                // Todo coup restrictions
                let coup_realign = state.legal_coup_realign(self.agent);
                if !state.has_effect(self.agent, Effect::CubanMissileCrisis) {
                    let coup_d = Decision::new(self.agent, Action::Coup, coup_realign.clone());
                    out.extend(coup_d.encode(state));
                }
                let realign_d = Decision::new(self.agent, Action::Realignment, coup_realign);
                out.extend(realign_d.encode(state));
                out
            },
            Action::CubanMissile => {
                // This is somewhat clunky, but should work
                // Todo include pass or empty
                let pass = Action::Pass.offset();
                let mut out = OutputVec::new(vec![pass]);
                let cuban_offset = self.action.offset();
                let remove = self.allowed.slice().iter().copied().map(|x| {
                    x + cuban_offset
                }).collect();
                out.extend(OutputVec::new(remove));
                out
            },
            Action::Coup if state.has_effect(self.agent, Effect::CubanMissileCrisis) => {
                // Todo include pass or empty?
                OutputVec::new(vec![Action::Pass.offset()])
            }
            _ => {
                let v = self.allowed.slice().iter().copied().map(|x|{
                    x + begin
                }).collect();
                OutputVec::new(v)
            }
        };
        out
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OutputIndex {
    data: usize,
}

impl OutputIndex {
    pub fn new(data: usize) -> OutputIndex {
        OutputIndex {data}
    }
    // Encode a single action-choice pair. 
    pub fn encode_single(action: Action, choice: usize, state: &GameState) -> Option<Self> {
        // Side shouldn't matter
        let d = Decision::new(state.side, action, vec![choice]);
        let vec = d.encode(state);
        let out = *vec.data().last()?;
        let decode = out.decode();
        if action == decode.0 && choice == decode.1 {
            Some(out)
        } else {
            None
        }
    }
    pub fn decode(&self) -> (Action, usize) {
        Action::action_from_offset(self.data)
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
        IndexMap {keys, values}
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
        let index = self.keys.binary_search_by_key(value, |k| {
            let i: usize = (*k).into();
            let v = self.values[i];
            v.unwrap()
        }).unwrap();
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
        // dbg!(output_vec.data);
        assert_eq!(output_vec.data.len(), 15 + 4);
        dbg!(output_vec.data.len());
        let mut events = 0;
        let mut spaces = 0;
        for out in output_vec.data() {
            let (action, _num) = out.decode();
            match action {
                Action::Event => events += 1,
                Action::Space => spaces += 1,
                _ => {},
            }
        }
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
        let mut inf: HashSet<_> = vec![Finland, Sweden, EGermany, Romania, Poland, Czechoslovakia,
            Austria, Hungary, Turkey, Syria, Lebanon, Israel, Iraq, Jordan, SaudiaArabia,
            GulfStates, Afghanistan, NKorea, SKorea].into_iter().map(|x| {
                x as usize
            }).collect();
        for out in output_vec.data() {
            let (action, num) = out.decode();
            match action {
                Action::Coup => coup += 1,
                Action::Realignment => realign += 1,
                Action::StandardOps => {
                    let in_set = inf.remove(&num);
                    assert!(in_set);
                }
                _ => {},
            }
        }
        assert_eq!(coup, 12);
        assert_eq!(realign, 12);

        assert_eq!(inf.len(), 0);
    }
}
