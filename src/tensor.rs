use crate::action::{Action, Decision, play_card_indices};
use crate::card::{self, Card};
use crate::state::GameState;

lazy_static! {
    static ref CARD_OFFSET: IndexMap<Card, usize> = {
        let mut sum = 0;
        let x = (0..card::NUM_CARDS + 1).filter_map(|c| {
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
                let space = state.legal_space(self.agent);
                let space_d = Decision::new(self.agent, Action::Space, space);
                let mut out = space_d.encode(state);
                let play_d = Decision::new(self.agent, Action::PlayCard, &[]);
                out.extend(play_d.encode(state));
                // Todo rarer things like discarding
                out
            },
            Action::PlayCard => {
                play_card_indices(state)
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

pub struct OutputIndex {
    data: usize,
}

impl OutputIndex {
    pub fn new(data: usize) -> OutputIndex {
        OutputIndex {data}
    }
    pub fn decode(&self, decision: Option<&Decision>, state: &GameState) -> (Action, usize) {
        todo!()
        // if let Some(dec) = decision {
        //     let action = decision.action; 
        //     (action, action.offset() + self.data)
        // } else {
        //     let action = match Action::action_index(self.data) {
        //         0 => Action::PlayCard,
        //         4 => Action::Space,

        //     }
        // }
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
            while k <= values.len() {
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
}
