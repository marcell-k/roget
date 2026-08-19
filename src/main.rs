const GAMES: &str = include_str!("../answers.txt");

use clap::{Parser, ValueEnum};
use roget::Guesser;
use std::time::Instant;

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
    PreCalc,
    Second,
    Weight,
    Prune,
    Cutoff,
    Popular,
    Cached,
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
        Implementation::PreCalc => {
            play(roget::algorithms::PreCalc::default, args.max);
        }
        Implementation::Second => {
            play(roget::algorithms::Second::default, args.max);
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
        Implementation::Cached => {
            play(roget::algorithms::Cached::default, args.max);
        }
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = roget::Wordle::default();
    let mut score = 0;
    let mut games = 0;
    let mut histogram = Vec::new();

    let start = Instant::now();

    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        games += 1;

        let guesser = mk();
        let answer = answer.as_bytes().try_into().expect("5 length");

        if let Some(s) = w.play(answer, guesser) {
            score += s;

            // Make sure histogram[s] exists.
            if s >= histogram.len() {
                histogram.resize(s + 1, 0);
            }

            histogram[s] += 1;
        } else {
            eprintln!("failed to guess, {}", std::str::from_utf8(&answer).unwrap());
        }
    }

    let sum: usize = histogram.iter().sum();

    if sum > 0 {
        for (score, count) in histogram.into_iter().enumerate().skip(1) {
            let frac = count as f64 / sum as f64;

            let filled = (30.0 * frac).round() as usize;
            let empty = 30 - filled;

            eprintln!(
                "{:>2}: {}{} ({})",
                score,
                "#".repeat(filled),
                " ".repeat(empty),
                count
            );
        }
    }

    let duration = start.elapsed();

    if games > 0 {
        println!("avg score: {:.2}", score as f64 / games as f64);
        println!("total time: {:?}", duration);
        println!("avg time per game: {:?}", duration / games as u32);
    } else {
        println!("no games played");
    }
}
