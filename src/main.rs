#![warn(rust_2018_idioms, clippy::all)]

#[macro_use]
mod macros;
mod error;
mod evaluator;
mod lexer;
mod object;
mod parser;

use crate::lexer::Lexer;
use crate::object::Object;
use crate::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let mut env = object::Environment::new();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let mut given_str = String::new();
                if line == ":{" {
                    loop {
                        let read_inner_line = rl.readline(">| ");
                        match read_inner_line {
                            Ok(inner_line) => {
                                if inner_line == ":}" {
                                    break;
                                } else {
                                    given_str += &inner_line;
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
                } else {
                    given_str += &line;
                }
                rl.add_history_entry(&given_str);
                let parsed = Parser::new(Lexer::new(&given_str)).parse_program();
                handle_error!(parsed => {
                    let object = evaluator::eval_program(parsed.unwrap(), &mut env);
                    handle_error!(object => print_object(object.unwrap()));
                });
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

fn print_object(obj: Object) {
    if let Object::DeclareVariable = obj {
        print!("");
    } else {
        println!("{}", obj);
    }
}
