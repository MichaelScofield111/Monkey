use std::io::{self, BufReader};
mod lexer;
mod repl;
mod token;

fn main() -> io::Result<()> {
    println!("Hello MichaelScofield! This is the Monkey programming language!");
    println!("Feel free to type in commands");

    let stdin = io::stdin();
    let stdout = io::stdout();

    let reader = BufReader::new(stdin);
    let writer = stdout;

    repl::start(reader, writer)?;
    Ok(())
}
