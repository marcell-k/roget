use crate::{Guess, Guesser};

pub struct Naive;
impl Default for Naive {
    fn default() -> Self {
        Naive
    }
}

impl Guesser for Naive {
    fn guess(&mut self, _history: &[Guess]) -> String {
        todo!()
    }
}
