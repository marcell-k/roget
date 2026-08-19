use once_cell::sync::OnceCell;

use crate::{Correctness, DICTIONARY, Guess, Guesser, MAX_MASK_ENUM, Word, enumerate_mask};
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static Word, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Cutoff {
    remaining: Cow<'static, [(&'static Word, usize)]>,
    patterns: Cow<'static, [[Correctness; 5]]>,
}

impl Default for Cutoff {
    fn default() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                let mut words = Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("every line is word + space + frequency");
                    let count: usize = count.parse().expect("every count is a number");
                    let word = word.as_bytes().try_into().expect("every word is 5 chars");
                    (word, count)
                }));

                words.sort_unstable_by_key(|&(_, count)| std::cmp::Reverse(count));
                words
            })),
            patterns: Cow::Borrowed(PATTERNS.get_or_init(|| Correctness::patterns().collect())),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    goodness: f64,
}

impl Guesser for Cutoff {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if let Some(last) = history.last() {
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
            self.patterns = Cow::Borrowed(PATTERNS.get().unwrap());
            return *b"tares";
        } else {
            assert!(!self.patterns.is_empty())
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut i = 0;
        let stop = (self.remaining.len() / 3).max(20);
        let mut best: Option<Candidate> = None;
        for &(word, count) in &*self.remaining {
            let mut totals = [0usize; MAX_MASK_ENUM];
            for (candidate, count) in &*self.remaining {
                let idx = enumerate_mask(&Correctness::compute(candidate, word));
                totals[idx] += count;
            }

            let sum: f64 = totals
                .into_iter()
                .filter(|t| *t != 0)
                .map(|t| {
                    // TODO: apply sigmoid
                    let p_of_this_pattern = t as f64 / remaining_count as f64;
                    p_of_this_pattern * p_of_this_pattern.log2()
                })
                .sum();
            let p_word = count as f64 / remaining_count as f64;
            let goodness = p_word * -sum;
            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness });
                }
            } else {
                best = Some(Candidate { word, goodness });
            }
            i += 1;
            if i >= stop {
                break;
            }
        }
        *best.unwrap().word
    }
}
