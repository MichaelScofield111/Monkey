use monkey::start;
use std::io::{self, BufReader};

fn main() -> io::Result<()> {
    println!("Hello MichaelScofield! This is the Monkey programming language!");
    println!("Feel free to type in commands");

    let stdin = io::stdin();
    let stdout = io::stdout();

    let reader = BufReader::new(stdin);
    let writer = stdout;

    start(reader, writer)?;
    Ok(())
}
