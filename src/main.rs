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
    OnceInit,
    PreCalc,
    Second,
    Weight,
    Prune,
    Cutoff,
    Popular,
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
        Implementation::OnceInit => {
            play(roget::algorithms::OnceInit::default, args.max);
        }
        Implementation::PreCalc => {
            play(roget::algorithms::PreCalc::default, args.max);
        }
        Implementation::Second => {
            play(roget::algorithms::Cached::default, args.max);
        }
        Implementation::Weight => {
            play(roget::algorithms::Weight::default, args.max);
        }
        Implementation::Prune => {
            play(roget::algorithms::Prune::default, args.max);
        }
        Implementation::Cutoff => {
            play(roget::algorithms::Cutoff::default, args.max);
        }
        Implementation::Popular => {
            play(roget::algorithms::Popular::default, args.max);
        }
    }
}

use std::time::Instant;

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = roget::Wordle::default();
    let mut score = 0;
    let mut games = 0;

    let start = Instant::now();

    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        games += 1;
        let guesser = (mk)();
        let answer = answer.as_bytes().try_into().expect("5 length");
        if let Some(s) = w.play(answer, guesser) {
            score += s;
            println!(
                "guessed '{}' in {}",
                std::str::from_utf8(&answer).expect("5 length"),
                s
            );
        } else {
            eprintln!("failed to guess");
        }
    }

    let duration = start.elapsed(); // Calculate total time

    println!("avg score: {:.2}", score as f64 / games as f64);
    println!("total time: {:?}", duration);
    if games > 0 {
        println!("avg time per game: {:?}", duration / games as u32);
    }
}
