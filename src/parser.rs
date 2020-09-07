use crate::error;
use crate::lexer::Lexer;
use crate::token::Token;

type Error = error::MonkeyErr;

// AST Types
pub type Program = Vec<Statement>;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    LetStatement { name: String, value: Expression },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    NoneVal,
    Ident(String),
}

#[derive(Debug)]
pub struct Parser {
    l: Vec<Token>,
    cur_position: usize,
}

macro_rules! expect_peek {
    ($e: expr => $p: pat | $e1: expr) => {
        if let $p = $e.take_token().1 {
            $e.next_token();
        } else {
            return Err(Error::ParseTokDiffErr {
                expected: $e1,
                got: $e.take_token().1.clone(),
            });
        }
    };
}

impl Parser {
    pub fn new(l: Lexer<'_>) -> Self {
        let l: Vec<Token> = l.collect();
        Self { l, cur_position: 0 }
    }

    fn next_token(&mut self) {
        self.cur_position += 1;
    }

    fn take_token(&self) -> (&Token, &Token) {
        let cur_tok = if self.cur_position >= self.l.len() {
            &Token::EOF
        } else {
            &self.l[self.cur_position]
        };
        let peek_tok = if self.cur_position >= self.l.len() - 1 {
            &Token::EOF
        } else {
            &self.l[self.cur_position + 1]
        };
        (cur_tok, peek_tok)
    }

    fn parse_program(&mut self) -> error::Result<Program> {
        let mut program: Program = vec![];
        while self.take_token().0 != &Token::EOF {
            program.push(self.parse_statement()?);
            self.next_token();
        }
        Ok(program)
    }

    fn parse_statement(&mut self) -> error::Result<Statement> {
        match self.take_token().0 {
            Token::LET => self.parse_let_stmt(),
            _ => unimplemented!(),
        }
    }

    fn parse_let_stmt(&mut self) -> error::Result<Statement> {
        expect_peek!(self => Token::IDENT(_) | Token::IDENT(String::new()));
        let name = self.take_token().0.unwrap_string();
        expect_peek!(self => Token::ASSIGN | Token::ASSIGN);

        // TODO: We're skipping the expressions until we
        // encounter a semicolon
        while self.take_token().0 != &Token::SEMICOLON {
            self.next_token();
        }

        Ok(Statement::LetStatement {
            name,
            value: Expression::NoneVal,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error;
    use crate::lexer::Lexer;
    #[test]
    fn parse_let() -> error::Result<()> {
        let input = r#"let x  5;
        let y = 10;
        let foobar = 838383;"#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![
            Statement::LetStatement {
                name: "x".to_string(),
                value: Expression::NoneVal,
            },
            Statement::LetStatement {
                name: "y".to_string(),
                value: Expression::NoneVal,
            },
            Statement::LetStatement {
                name: "foobar".to_string(),
                value: Expression::NoneVal,
            },
        ];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }
}
