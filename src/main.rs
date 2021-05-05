#![warn(rust_2018_idioms, clippy::all)]

mod error;
mod lexer;
mod parser;
mod repl;

fn main() {
    repl::start_repl();
}
