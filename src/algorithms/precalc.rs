use once_cell::sync::OnceCell;

use crate::{Correctness, DICTIONARY, Guess, Guesser, Word};
use std::{borrow::Cow, collections::BTreeMap};

static INITIAL: OnceCell<Vec<(&'static Word, usize)>> = OnceCell::new();

type MatchKey = (Word, Word, [Correctness; 5]);
static MATCH: OnceCell<BTreeMap<MatchKey, bool>> = OnceCell::new();

pub struct PreCalc {
    remaining: Cow<'static, [(&'static Word, usize)]>,
}

impl Default for PreCalc {
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
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    goodness: f64,
}

impl Guesser for PreCalc {
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
            return *b"crate";
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut best: Option<Candidate> = None;
        for &(word, _) in &*self.remaining {
            let mut sum = 0.0;
            // TODO: dont consider correctness patterns that had no candidates in the prev iter
            for pattern in Correctness::patterns() {
                // considering a world where we _did_ guess `word` and got `pattern` as the
                // correctness. now, compute what _then_ is left.
                let mut in_pattern_total = 0;
                for &(candidate, count) in &*self.remaining {
                    let matches = MATCH.get_or_init(|| {
                        let words = &INITIAL.get().unwrap()[..512];
                        let mut out = BTreeMap::new();

                        for &(word1, _) in words {
                            for &(word2, _) in words {
                                if word2 < word1 {
                                    break;
                                }
                                for pattern in Correctness::patterns() {
                                    let g = Guess {
                                        word: Cow::Borrowed(word1),
                                        mask: pattern,
                                    };
                                    out.insert((*word1, *word2, pattern), g.matches(candidate));
                                }
                            }
                        }
                        out
                    });

                    let key = if word < candidate {
                        (*word, *candidate, pattern)
                    } else {
                        (*candidate, *word, pattern)
                    };
                    if matches.get(&key).copied().unwrap_or_else(|| {
                        let g = Guess {
                            word: Cow::Borrowed(word),
                            mask: pattern,
                        };
                        g.matches(candidate)
                    }) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    continue;
                }
                // TODO: apply sigmoid
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
            }
            // TODO: weight this by p_word
            let goodness = -sum;
            if let Some(c) = best {
                // Is this one better?
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
