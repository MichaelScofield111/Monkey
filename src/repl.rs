use std::io::{self, BufRead, Write};

use crate::{ast::Node, lexer::Lexer, parser::Parser};

const PROMPT: &str = ">>";

const MONKEY_FACE: &str = r#"            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"""""""-./, -' /
   ''-' /_   ^ ^   _\ '-''
       |  \._   _./  |
       \   \ '~' /   /
        '._ '-=-' _.'
           '-----'
"#;

pub fn start<R: BufRead, W: Write>(mut input: R, mut output: W) -> io::Result<()> {
    writeln!(output, "{}", MONKEY_FACE)?;

    let mut line = String::new();
    loop {
        write!(output, "{}", PROMPT)?;
        output.flush()?;

        line.clear();
        let bytes = input.read_line(&mut line)?;
        if bytes == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }

        let l = Lexer::new(&line);
        let mut p = Parser::new(l);
        let program = p.parse_program();

        if !p.errors.is_empty() {
            print_errors(&mut output, &p.errors)?;
            continue;
        }

        writeln!(output, "{}", program.string())?;
    }
}

fn print_errors<W: Write>(output: &mut W, errs: &[String]) -> io::Result<()> {
    for msg in errs {
        writeln!(output, "\t{}", msg)?;
    }
    Ok(())
}
