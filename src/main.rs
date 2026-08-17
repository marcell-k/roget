const GAMES: &str = include_str!("../answers.txt");

fn main() {
    let w = roget::Worlde::default();
    for answer in GAMES.split_whitespace() {
        let guesser = roget::algorithms::Naive;
        w.play(answer, guesser);
    }
    println!("Hello, world!");
}
