#![allow(unused)]
use std::collections::HashMap;

use crate::error;
use crate::lexer::Lexer;
use crate::token::Token;

type Error = error::MonkeyErr;

// AST Types
pub type Program = Vec<Statement>;
type PrefixParseFn = fn() -> Expression;
type InfixParseFn = fn(&Expression) -> Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    LetStmt { name: String, value: Expression },
    ReturnStmt { value: Expression },
    ExpressionStmt { expression: Expression },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    NoneVal,
    Ident(String),
}

pub struct Parser {
    l: Vec<Token>,
    cur_position: usize,
    prefix_fns: HashMap<Token, PrefixParseFn>,
    infix_fns: HashMap<Token, InfixParseFn>,
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
        let prefix_fns: HashMap<Token, PrefixParseFn> = HashMap::new();
        let infix_fns: HashMap<Token, InfixParseFn> = HashMap::new();
        Self {
            l,
            cur_position: 0,
            prefix_fns,
            infix_fns,
        }
    }

    fn register_prefix(&mut self, tok: &Token, f: &PrefixParseFn) {
        self.prefix_fns.insert(tok.clone(), *f);
    }

    fn register_infix(&mut self, tok: &Token, f: &InfixParseFn) {
        self.infix_fns.insert(tok.clone(), *f);
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
            Token::RETURN => self.parse_return_stmt(),
            _ => self.parse_expression_stmt(),
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

        Ok(Statement::LetStmt {
            name,
            value: Expression::NoneVal,
        })
    }

    fn parse_return_stmt(&mut self) -> error::Result<Statement> {
        self.next_token();
        // TODO: We're skipping the expressions until we
        // encounter a semicolon
        while self.take_token().0 != &Token::SEMICOLON {
            self.next_token();
        }
        Ok(Statement::ReturnStmt {
            value: Expression::NoneVal,
        })
    }

    fn parse_expression_stmt(&mut self) -> error::Result<Statement> {
        unimplemented!();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error;
    use crate::lexer::Lexer;
    #[test]
    fn parse_let() -> error::Result<()> {
        let input = r#"
        let x = 5;
        let y = 10;
        let foobar = 838383;
        "#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![
            Statement::LetStmt {
                name: "x".to_string(),
                value: Expression::NoneVal,
            },
            Statement::LetStmt {
                name: "y".to_string(),
                value: Expression::NoneVal,
            },
            Statement::LetStmt {
                name: "foobar".to_string(),
                value: Expression::NoneVal,
            },
        ];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }

    #[test]
    fn parse_return() -> error::Result<()> {
        let input = r#"
        return 5;
        return 10;
        return 993322;
        "#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![
            Statement::ReturnStmt {
                value: Expression::NoneVal,
            },
            Statement::ReturnStmt {
                value: Expression::NoneVal,
            },
            Statement::ReturnStmt {
                value: Expression::NoneVal,
            },
        ];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }

    #[test]
    fn parse_ident() -> error::Result<()> {
        let input = r#"
        foobar;
        "#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![Statement::ExpressionStmt {
            expression: Expression::NoneVal,
        }];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }
}
