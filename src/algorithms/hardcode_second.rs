//! Place at src/algorithms/cached.rs
//! Register in src/algorithms.rs:
//!   mod cached;
//!   pub use cached::Cached;
//! Register in src/main.rs enum Implementation + match arm (see below).

use once_cell::sync::OnceCell;

use crate::{Correctness, DICTIONARY, Guess, Guesser, Word};
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static Word, usize)>> = OnceCell::new();

const ALL_WRONG: [Correctness; 5] = [Correctness::Wrong; 5];

const TARES_ALL_WRONG_BEST: &Word = b"could";
const CRATE_ALL_WRONG_BEST: &Word = b"sound";

pub struct Cached {
    remaining: Cow<'static, [(&'static Word, usize)]>,
}

impl Default for Cached {
    fn default() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("every line is word + space + frequency");
                    let count: usize = count.parse().expect("every count is a number");
                    let word = word.as_bytes().try_into().expect("every word is 5 chars");
                    (word, count)
                }))
            })),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    goodness: f64,
}

impl Guesser for Cached {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if let Some(last) = history.last() {
            // fast path: first guess was tares/crate and came back all-Wrong ->
            // skip the O(n^2) entropy scan, use precomputed best second guess.
            if history.len() == 1 && last.mask == ALL_WRONG {
                if &*last.word == b"tares" {
                    return *TARES_ALL_WRONG_BEST;
                }
                if &*last.word == b"crate" {
                    return *CRATE_ALL_WRONG_BEST;
                }
            }

            if matches!(self.remaining, Cow::Owned(_)) {
                self.remaining
                    .to_mut()
                    .retain(|(word, _)| last.matches(word));
            } else {
                self.remaining = Cow::Owned(
                    self.remaining
                        .iter()
                        .filter(|(word, _)| last.matches(word))
                        .copied()
                        .collect(),
                );
            }
        }
        if history.is_empty() {
            return *b"tares";
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut best: Option<Candidate> = None;
        for (word, _) in &*self.remaining {
            let mut sum = 0.0;
            for pattern in Correctness::patterns() {
                let mut in_pattern_total = 0;
                for (candidate, count) in &*self.remaining {
                    let g = Guess {
                        word: Cow::Borrowed(*word),
                        mask: pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    continue;
                }
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
            }
            let goodness = -sum;
            if let Some(c) = best {
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness });
                }
            } else {
                best = Some(Candidate { word, goodness });
            }
        }
        *best.unwrap().word
    }
}
