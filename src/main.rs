const GAMES: &str = include_str!("../answers.txt");

use clap::{Parser, ValueEnum};
use roget::Guesser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    implementation: Implementation,

    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
enum Implementation {
    Allocs,
    Vecrem,
}

fn main() {
    let args = Args::parse();
    match args.implementation {
        Implementation::Allocs => {
            play(roget::algorithms::Allocs::default, args.max);
        }
        Implementation::Vecrem => {
            play(roget::algorithms::Vecrem::default, args.max);
        }
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = roget::Wordle::default();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        let answer = answer.as_bytes().try_into().expect("5 length");
        if let Some(score) = w.play(answer, guesser) {
            println!(
                "guessed '{}' in {}",
                std::str::from_utf8(&answer).expect("5 length"),
                score
            );
        } else {
            eprintln!("failed to guess");
        }
    }
}
