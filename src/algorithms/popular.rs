use crate::{DICTIONARY, Guess, Guesser, Word};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static Word, usize)>> = OnceCell::new();

/// A strawman algorithm which simply chooses the most popular word of the
/// words remaining which match the most recent mask
pub struct Popular {
    remaining: Cow<'static, [(&'static Word, usize)]>,
}

impl Default for Popular {
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

impl Guesser for Popular {
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
            *b"tares"
        } else {
            *self.remaining.first().unwrap().0
        }
    }
}
