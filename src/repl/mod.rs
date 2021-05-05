use crate::lexer::Lexer;
use crate::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn start_repl() {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
            Ok(line) => {
                let mut given_str = String::new();
                match line.as_str() {
                    ":quit" => break,
                    ":{" => multiline_reading(&mut rl, &mut given_str),
                    _ => given_str += &line,
                }
                rl.add_history_entry(&given_str);
                if !given_str.is_empty() {
                    let mut parser = Parser::new(Lexer::new(&given_str));
                    let program = parser.parse_program();
                    println!("{:#?}", program);
                }
            }
        }
    }
}

fn multiline_reading(rl: &mut Editor<()>, given_str: &mut String) {
    loop {
        let read_inner_line = rl.readline(">| ");
        match read_inner_line {
            Ok(inner_line) => {
                if inner_line == ":}" {
                    break;
                } else {
                    *given_str += "\n";
                    *given_str += &inner_line;
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
