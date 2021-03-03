#![warn(rust_2018_idioms, clippy::all)]

#[macro_use]
mod macros;
mod code;
mod compiler;
mod error;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod vm;

use crate::evaluator::{gc::GCBox, Evaluator};
use crate::lexer::Lexer;
use crate::object::{Environment, Object};
use crate::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let env = Rc::new(RefCell::new(Environment::new()));
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let mut given_str = String::new();
                if line == ":quit" {
                    break;
                } else if line == ":{" {
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
                if !given_str.is_empty() {
                    let parsed = Parser::new(Lexer::new(&given_str)).parse_program();
                    handle_error!(parsed => {
                        let mut eval = Evaluator::new();
                        let object = eval.eval(parsed.unwrap(), &env);
                        handle_error!(object => print_object(object.unwrap()));
                    });
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
}

fn print_object(obj: GCBox<Object>) {
    if let Object::DeclareVariable = *obj {
        print!("");
    } else {
        println!("{:?}", *obj);
    }
}
