use std::io::{self, BufRead, Write};

use crate::{lexer::Lexer, token::TokenType};

const PROMPT: &str = ">>";

pub fn start<R: BufRead, W: Write>(mut input: R, mut output: W) -> io::Result<()> {
    let mut line = String::new();

    loop {
        write!(output, "{}", PROMPT)?;
        output.flush()?;

        line.clear();
        let bytes = input.read_line(&mut line)?;
        if bytes == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }

        let mut l = Lexer::new(&line);
        loop {
            let token = l.next_token();
            if token.r#type == TokenType::EOF {
                break;
            }

            writeln!(output, "{:?}", token)?;
        }
    }
}
