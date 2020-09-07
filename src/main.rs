#![warn(rust_2018_idioms)]

mod lexer;
mod parser;
mod token;
mod error;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let lex = lexer::Lexer::new(&line);
                println!("{:?}", lex.collect::<Vec<token::Token>>())
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
