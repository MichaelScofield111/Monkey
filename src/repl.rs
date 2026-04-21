use std::{
    fs,
    io::{self, BufRead, Write},
    path::Path,
};

use crate::{
    ast::Program, environment::Environment, eval::eval, lexer::Lexer, object::Object,
    parser::Parser,
};

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
    let mut env = Environment::new();
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
        match eval(&program, &mut env) {
            Ok(obj) => {
                if !matches!(obj, Object::Null(_)) {
                    writeln!(output, "{}", obj.inspect())?;
                }
            }
            Err(e) => {
                writeln!(output, "eval error: {}", e)?;
            }
        }
    }
}

fn print_errors<W: Write>(output: &mut W, errs: &[String]) -> io::Result<()> {
    for msg in errs {
        writeln!(output, "\t{}", msg)?;
    }
    Ok(())
}

pub fn run_file<P: AsRef<Path>, W: Write, E: Write>(
    path: P,
    mut output: W,
    mut err_output: E,
) -> io::Result<()> {
    let content = fs::read_to_string(path)?;

    let l = Lexer::new(&content);
    let mut p = Parser::new(l);
    let program = p.parse_program();

    if !p.errors.is_empty() {
        for msg in &p.errors {
            writeln!(err_output, "{}", msg)?;
        }
        return Err(io::Error::new(io::ErrorKind::InvalidData, "parse error"));
    }

    let mut env = Environment::new();
    for stmt in &program.statements {
        let one_stmt_program = Program {
            statements: vec![stmt.clone()],
        };

        match eval(&one_stmt_program, &mut env) {
            Ok(obj) => {
                if !matches!(obj, Object::Null(_)) {
                    writeln!(output, "{}", obj.inspect())?;
                }
            }
            Err(e) => {
                writeln!(err_output, "eval err: {}", e)?;
                return Err(io::Error::other("eval error"));
            }
        }
    }

    Ok(())
}
