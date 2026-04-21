use clap::{CommandFactory, Parser};
use monkey::{run_file, start};
use std::io::{self, BufReader};

#[derive(Parser, Debug)]
#[command(name = "monkey", about = "Monkey programming language interpreter")]
struct Args {
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    #[arg(short = 'f', long = "file")]
    file: Option<String>,

    #[arg()]
    extras: Vec<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if !args.extras.is_empty() {
        let mut cmd = Args::command();
        cmd.print_help()?;
        println!();
        return Ok(());
    }

    if args.interactive || args.file.is_none() {
        println!("Hello MichaelScofield! This is the Monkey programming language!");
        println!("Feel free to type in commands");

        let stdin = io::stdin();
        let stdout = io::stdout();

        let reader = BufReader::new(stdin);
        let writer = stdout;

        start(reader, writer)?;
        return Ok(());
    }

    if let Some(path) = args.file {
        let stdout = io::stdout();
        let stderr = io::stderr();
        run_file(path, stdout, stderr)?;
    }

    Ok(())
}
