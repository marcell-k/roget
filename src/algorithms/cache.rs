use crate::{Correctness, DICTIONARY, Guess, Guesser, MAX_MASK_ENUM, Word, enumerate_mask};
use once_cell::sync::OnceCell;
use once_cell::unsync::OnceCell as UnsyncOnceCell;
use std::borrow::Cow;
use std::cell::Cell;
use std::num::NonZeroU8;

static INITIAL: OnceCell<Vec<(&'static Word, f64, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

const fn count_lines(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut lines = 0;
    let mut saw_content_on_line = false;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            lines += 1;
            saw_content_on_line = false;
        } else {
            saw_content_on_line = true;
        }
        i += 1;
    }
    if saw_content_on_line {
        lines += 1;
    }
    lines
}

const NUM_WORDS: usize = count_lines(DICTIONARY);

#[derive(Debug, Copy, Clone)]
struct CacheValue(NonZeroU8);

impl CacheValue {
    fn new(val: u8) -> Self {
        Self(NonZeroU8::new(val + 1).unwrap())
    }

    fn get(&self) -> u8 {
        self.0.get() - 1
    }
}

struct Cache(Vec<Cell<Option<CacheValue>>>);

impl Cache {
    fn row(&self, idx: usize) -> &[Cell<Option<CacheValue>>] {
        &self.0[idx * NUM_WORDS..(idx + 1) * NUM_WORDS]
    }
}

impl Default for Cache {
    fn default() -> Self {
        let mut cells = Vec::with_capacity(NUM_WORDS * NUM_WORDS);
        cells.resize_with(NUM_WORDS * NUM_WORDS, || Cell::new(None));
        Cache(cells)
    }
}

thread_local! {
    static COMPUTES: UnsyncOnceCell<Box<Cache>> = Default::default();
}

pub struct Cached {
    remaining: Cow<'static, [(&'static Word, f64, usize)]>,
    patterns: Cow<'static, [[Correctness; 5]]>,
    entropy: Vec<f64>,
}

impl Default for Cached {
    fn default() -> Self {
        Self::new()
    }
}

fn est_steps_left(entropy: f64) -> f64 {
    (entropy * 3.870 + 3.679).ln()
}
const PRINT_ESTIMATION: bool = false;

const L: f64 = 1.0;
const K: f64 = 30000000.0;
const X0: f64 = 0.00000497;

fn sigmoid(p: f64) -> f64 {
    L / (1.0 + (-K * (p - X0)).exp())
}
const PRINT_SIGMOID: bool = false;

impl Cached {
    pub fn new() -> Self {
        let remaining = Cow::Borrowed(
            INITIAL
                .get_or_init(|| {
                    let words: Vec<(&'static Word, usize, usize)> = DICTIONARY
                        .lines()
                        .enumerate()
                        .map(|(idx, line)| {
                            let (word, count) = line
                                .split_once(' ')
                                .expect("every line is word + space + frequency");
                            let count: usize = count.parse().expect("every count is a number");
                            let word: &'static Word =
                                word.as_bytes().try_into().expect("every word is 5 chars");
                            (word, count, idx)
                        })
                        .collect();

                    let sum: usize = words.iter().map(|&(_, count, _)| count).sum();

                    if PRINT_SIGMOID {
                        for &(word, count, _) in &words {
                            let p = count as f64 / sum as f64;
                            println!(
                                "{} {:.6}% -> {:.6}% ({})",
                                std::str::from_utf8(word).unwrap(),
                                100.0 * p,
                                100.0 * sigmoid(p),
                                count
                            );
                        }
                    }

                    words
                        .into_iter()
                        .map(|(word, count, idx)| (word, sigmoid(count as f64 / sum as f64), idx))
                        .collect()
                })
                .as_slice(),
        );

        COMPUTES.with(|c| {
            let _ = c.get_or_init(Box::default);
        });

        Self {
            remaining,
            patterns: Cow::Borrowed(
                PATTERNS
                    .get_or_init(|| Correctness::patterns().collect())
                    .as_slice(),
            ),
            entropy: Vec::new(),
        }
    }

    pub fn finish(&self, guesses: usize) {
        if PRINT_ESTIMATION {
            for (i, &entropy) in self.entropy.iter().enumerate() {
                let guesses_needed = guesses - (i + 1);
                println!("{} {}", entropy, guesses_needed);
            }
        }
    }
}

#[inline]
fn get_correctness_packed(
    row: &[Cell<Option<CacheValue>>],
    guess: &Word,
    answer: &Word,
    answer_idx: usize,
) -> u8 {
    let cell = &row[answer_idx];
    match cell.get() {
        Some(a) => a.get(),
        None => {
            let correctness = enumerate_mask(&Correctness::compute(answer, guess)) as u8;
            cell.set(Some(CacheValue::new(correctness)));
            correctness
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    e_score: f64,
}

impl Guesser for Cached {
    fn guess(&mut self, history: &[Guess]) -> Word {
        let score = history.len() as f64;

        if let Some(last) = history.last() {
            let reference = enumerate_mask(&last.mask) as u8;
            let last_idx = self
                .remaining
                .iter()
                .find(|(word, _, _)| &*last.word == *word)
                .unwrap()
                .2;
            COMPUTES.with(|c| {
                let row = c.get().unwrap().row(last_idx);
                if matches!(self.remaining, Cow::Owned(_)) {
                    self.remaining.to_mut().retain(|(word, _, word_idx)| {
                        reference == get_correctness_packed(row, &last.word, word, *word_idx)
                    });
                } else {
                    self.remaining = Cow::Owned(
                        self.remaining
                            .iter()
                            .filter(|(word, _, word_idx)| {
                                reference
                                    == get_correctness_packed(row, &last.word, word, *word_idx)
                            })
                            .copied()
                            .collect(),
                    );
                }
            });
        }
        if history.is_empty() {
            self.patterns = Cow::Borrowed(PATTERNS.get().unwrap().as_slice());
            return *b"tares";
        } else {
            assert!(!self.patterns.is_empty());
        }

        let remaining_p: f64 = self.remaining.iter().map(|&(_, p, _)| p).sum();
        let remaining_entropy = -self
            .remaining
            .iter()
            .map(|&(_, p, _)| {
                let p = p / remaining_p;
                p * p.log2()
            })
            .sum::<f64>();
        self.entropy.push(remaining_entropy);

        let mut best: Option<Candidate> = None;
        let mut i = 0;
        let stop = (self.remaining.len() / 3).max(20);
        for &(word, p_weight, word_idx) in &*self.remaining {
            let mut totals = [0.0f64; MAX_MASK_ENUM];

            COMPUTES.with(|c| {
                let row = c.get().unwrap().row(word_idx);
                for &(candidate, p, candidate_idx) in &*self.remaining {
                    let idx = get_correctness_packed(row, word, candidate, candidate_idx);
                    totals[usize::from(idx)] += p;
                }
            });

            let sum: f64 = totals
                .into_iter()
                .filter(|t| *t != 0.0)
                .map(|p| {
                    let p_of_this_pattern = p / remaining_p;
                    p_of_this_pattern * p_of_this_pattern.log2()
                })
                .sum();

            let p_word = p_weight / remaining_p;
            let e_info = -sum;
            let e_score = p_word * (score + 1.0)
                + (1.0 - p_word) * (score + est_steps_left(remaining_entropy - e_info));
            if let Some(c) = best {
                if e_score < c.e_score {
                    best = Some(Candidate { word, e_score });
                }
            } else {
                best = Some(Candidate { word, e_score });
            }

            i += 1;
            if i >= stop {
                break;
            }
        }
        *best.unwrap().word
    }
}
