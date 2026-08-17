use std::collections::HashSet;
pub mod algorithms;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Worlde {
    pub dictionary: HashSet<&'static str>,
}
impl Default for Worlde {
    fn default() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("every line is a word + space + freq")
                    .0
            })),
        }
    }
}
impl Worlde {
    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();
        for i in 1..32 {
            let guess = guesser.guess(&history);
            if guess == answer {
                return Some(i);
            }
            debug_assert!(self.dictionary.contains(guess.as_str()));
            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: guess,
                mask: correctness,
            });
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    Correct,
    Misplaced,
    Wrong,
}
impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);

        let mut c = [Correctness::Wrong; 5];
        // Correct
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }
        // Misplaced
        let mut used = [false; 5];
        for (i, &c) in c.iter().enumerate() {
            if c == Correctness::Correct {
                used[i] = true;
            }
        }
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                // Already marked
                continue;
            }
            if answer.chars().enumerate().any(|(i, a)| {
                if a == g && !used[i] {
                    used[i] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }
        c
    }
}

pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}
impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

#[cfg(test)]
mod tests {
    mod game {
        use crate::{Guess, Worlde};

        #[test]
        fn play() {
            let w = Worlde::default();
            fn guess(_history: &[Guess]) -> String {
                "moved".to_string()
            }
            assert_eq!(w.play("moved", guess as fn(&[Guess]) -> String), Some(1));
        }
        #[test]
        fn two_guesses() {
            let w = Worlde::default();
            fn guess(history: &[Guess]) -> String {
                if history.len() == 1 {
                    return "right".to_string();
                }
                "wrong".to_string()
            }
            assert_eq!(w.play("right", guess as fn(&[Guess]) -> String), Some(2));
        }
    }
    mod compute {
        use crate::Correctness;

        macro_rules! mask{
            (C) => { Correctness::Correct};
            (M) => { Correctness::Misplaced};
            (W) => { Correctness::Wrong};
            ($($c: tt)+) => {[
                $(mask!($c)),+
                ]}
        }

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcdi", "abcdi"), mask![C C C C C])
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcdi", "idbca"), mask![M M M M M])
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcdi", "fghjk"), mask![W W W W W])
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask![C C W W W])
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask![W W M M W])
        }
        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask![W C M W W])
        }
        #[test]
        fn test1() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask![C M W W W])
        }
    }
}
