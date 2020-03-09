use crate::action::{Action, Decision};
use crate::card::{self, Card};

use std::marker::PhantomData;

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
    fn encode(&self) -> OutputVec;
}

impl TensorOutput for Decision {
    fn encode(&self) -> OutputVec {
        let vec = match self.action {
            Action::Event(c) => {
                let begin = self.action.offset();
                if let Some(card) = c {
                    // We know what event we're playing, just not precisely how
                    let off = *CARD_OFFSET.get(card).unwrap() + begin;
                    self.allowed.slice().iter().copied().map(|x| {
                        x + off
                    }).collect()
                } else {
                    // We just want to enumerate cards we can event
                    self.allowed.slice().iter().copied().map(|x| {
                        x + begin
                    }).collect()
                }
            }
            Action::PlayCard => unimplemented!(),
            _ => todo!()
        };
        OutputVec::new(vec)
    }
}

pub struct OutputIndex {
    data: usize,
}

impl OutputIndex {
    pub fn new(data: usize) -> OutputIndex {
        OutputIndex {data}
    }
    pub fn decode(&self) -> (Action, usize) {
        todo!()
    }
}

pub struct IndexMap<K, V> {
    values: Vec<Option<V>>,
    phantom: PhantomData<K>,
}

impl<K: Into<usize>, V> IndexMap<K, V> {
    pub fn new(pairs: Vec<(K, V)>) -> Self {
        let mut values = Vec::new();
        for (k, v) in pairs.into_iter() {
            let k: usize = k.into();
            while k <= values.len() {
                values.push(None);
            }
            values[k] = Some(v);
        }
        IndexMap {values, phantom: PhantomData}
    }
    pub fn get(&self, key: K) -> Option<&V> {
        let key = key.into();
        if key >= self.values.len() {
            None
        } else {
            self.values[key].as_ref()
        }
    }
}
