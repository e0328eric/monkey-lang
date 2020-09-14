#![warn(rust_2018_idioms, clippy::all)]

mod error;
mod evaluator;
mod lexer;
mod object;
mod parser;

use crate::lexer::Lexer;
use crate::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() -> error::Result<()> {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let parsed = Parser::new(Lexer::new(&line)).parse_program();
                if let Err(error) = parsed {
                    eprintln!("{}", error);
                } else {
                    let objects = evaluator::eval_program(parsed.unwrap());
                    if let Err(error) = objects {
                        eprintln!("{}", error);
                    } else {
                        println!("{}", objects.unwrap());
                    }
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
