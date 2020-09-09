#![warn(rust_2018_idioms)]

mod error;
mod lexer;
mod parser;
mod token;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() -> error::Result<()> {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let lex = lexer::Lexer::new(&line);
                let mut parser = parser::Parser::new(lex);
                println!("{:#?}", parser.parse_program()?);
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
