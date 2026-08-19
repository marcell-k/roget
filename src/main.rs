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
    Naive,
    Allocs,
}

fn main() {
    let args = Args::parse();
    match args.implementation {
        Implementation::Naive => {
            play(roget::algorithms::Naive::default, args.max);
        }
        Implementation::Allocs => {
            play(roget::algorithms::Allocs::default, args.max);
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
        if let Some(score) = w.play(answer, guesser) {
            println!("guessed '{}' in {}", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}
