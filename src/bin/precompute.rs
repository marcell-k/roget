use roget::{Correctness, Guess, Word};
use std::borrow::Cow;

const DICTIONARY: &str = include_str!("../../dictionary.txt");

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    goodness: f64,
}

fn best_second_guess(first: &'static Word) -> &'static Word {
    let all: Vec<(&'static Word, usize)> = DICTIONARY
        .lines()
        .map(|line| {
            let (word, count) = line
                .split_once(' ')
                .expect("every line is word + space + frequency");
            let count: usize = count.parse().expect("every count is a number");
            let word: &'static Word = word.as_bytes().try_into().expect("every word is 5 chars");
            (word, count)
        })
        .collect();

    // filter to words consistent with `first` guessed and getting all-Wrong
    let all_wrong = [Correctness::Wrong; 5];
    let g = Guess {
        word: Cow::Borrowed(first),
        mask: all_wrong,
    };
    let remaining: Vec<(&'static Word, usize)> = all
        .iter()
        .copied()
        .filter(|(candidate, _)| g.matches(candidate))
        .collect();

    let remaining_count: usize = remaining.iter().map(|&(_, c)| c).sum();

    let mut best: Option<Candidate> = None;
    for &(word, _) in &remaining {
        let mut sum = 0.0;
        for pattern in Correctness::patterns() {
            let mut in_pattern_total = 0;
            for &(candidate, count) in &remaining {
                let g = Guess {
                    word: Cow::Borrowed(word),
                    mask: pattern,
                };
                if g.matches(candidate) {
                    in_pattern_total += count;
                }
            }
            if in_pattern_total == 0 {
                continue;
            }
            let p = in_pattern_total as f64 / remaining_count as f64;
            sum += p * p.log2();
        }
        let goodness = -sum;
        if best.is_none_or(|c| goodness > c.goodness) {
            best = Some(Candidate { word, goodness });
        }
    }
    best.unwrap().word
}

fn main() {
    let tares: &'static Word = b"tares";
    let crate_: &'static Word = b"crate";

    let best_after_tares = best_second_guess(tares);
    let best_after_crate = best_second_guess(crate_);

    println!(
        "tares all-wrong -> best second guess: {}",
        std::str::from_utf8(best_after_tares).unwrap()
    );
    println!(
        "crate all-wrong -> best second guess: {}",
        std::str::from_utf8(best_after_crate).unwrap()
    );
}
