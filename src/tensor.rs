use crate::action::{Action, Decision};

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
            Action::Event(_) => todo!(),
            Action::PlayCard => unimplemented!(),
            _ => todo!()
        };

        // let offset = self.action.offset();
        // let vec = self.allowed.slice().iter().map(|x| *x + offset).collect();
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
