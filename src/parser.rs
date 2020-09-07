#![allow(unused)]
use std::collections::HashMap;

use crate::error;
use crate::lexer::Lexer;
use crate::token::Token;

type Error = error::MonkeyErr;

// AST Types
pub type Program = Vec<Statement>;
type PrefixParseFn = fn(&mut Parser) -> error::Result<Expression>;
type InfixParseFn = fn(&mut Parser, &Expression) -> error::Result<Expression>;

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
    Integer(i64),
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Precedence {
    LOWEST,
    EQUALS,
    LESSGREATER,
    SUM,
    PRODUCT,
    PREFIX,
    CALL,
}

fn take_precedence(tok: &Token) -> Precedence {
    match tok {
        Token::EQ => Precedence::EQUALS,
        Token::NOTEQ => Precedence::EQUALS,
        Token::LT => Precedence::LESSGREATER,
        Token::GT => Precedence::LESSGREATER,
        Token::PLUS => Precedence::SUM,
        Token::MINUS => Precedence::SUM,
        Token::ASTERISK => Precedence::PRODUCT,
        Token::SLASH => Precedence::PRODUCT,
        _ => Precedence::LOWEST,
    }
}

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

    pub fn new(l: Lexer<'_>) -> Self {
        let l: Vec<Token> = l.collect();
        Self { l, cur_position: 0 }
    }

    fn prefix_fn(&mut self) -> Option<PrefixParseFn> {
        match self.take_token().0 {
            Token::IDENT(_) => Some(Parser::parse_identifier),
            Token::INT(_) => Some(Parser::parse_integer),
            Token::BANG => Some(Parser::parse_prefix_expr),
            Token::MINUS => Some(Parser::parse_prefix_expr),
            _ => None,
        }
    }

    fn infix_fn(&mut self) -> Option<InfixParseFn> {
        match self.take_token().1 {
            Token::PLUS => Some(Parser::parse_infix_expr),
            Token::MINUS => Some(Parser::parse_infix_expr),
            Token::ASTERISK => Some(Parser::parse_infix_expr),
            Token::SLASH => Some(Parser::parse_infix_expr),
            Token::EQ => Some(Parser::parse_infix_expr),
            Token::NOTEQ => Some(Parser::parse_infix_expr),
            Token::LT => Some(Parser::parse_infix_expr),
            Token::GT => Some(Parser::parse_infix_expr),
            _ => None,
        }
    }

    fn parse_identifier(&mut self) -> error::Result<Expression> {
        if let Token::IDENT(ref i) = self.take_token().0 {
            Ok(Expression::Ident(i.to_string()))
        } else {
            Err(Error::ParseExprErr {
                expected: "identifier".to_string(),
                got: self.take_token().0.clone(),
            })
        }
    }

    fn parse_integer(&mut self) -> error::Result<Expression> {
        if let Token::INT(i) = self.take_token().0 {
            Ok(Expression::Integer(*i))
        } else {
            Err(Error::ParseExprErr {
                expected: "integer".to_string(),
                got: self.take_token().0.clone(),
            })
        }
    }

    fn parse_prefix_expr(&mut self) -> error::Result<Expression> {
        let operator = self.take_token().0.clone();
        self.next_token();

        let right = Box::new(self.parse_expression(Precedence::PREFIX)?);
        Ok(Expression::Prefix { operator, right })
    }

    fn parse_infix_expr(&mut self, left: &Expression) -> error::Result<Expression> {
        let operator = self.take_token().0.clone();
        let precedence = take_precedence(self.take_token().0);
        self.next_token();

        let right = Box::new(self.parse_expression(precedence)?);
        Ok(Expression::Infix {
            left: Box::new(left.clone()),
            operator,
            right,
        })
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
        let expression = self.parse_expression(Precedence::LOWEST)?;
        while self.take_token().0 != &Token::SEMICOLON {
            self.next_token();
        }
        Ok(Statement::ExpressionStmt { expression })
    }

    fn parse_expression(&mut self, prece: Precedence) -> error::Result<Expression> {
        let mut left_exp = if let Some(prefix) = self.prefix_fn() {
            prefix(self)?
        } else {
            return Err(Error::PrefixNoneErr);
        };

        while self.take_token().1 != &Token::SEMICOLON
            && prece < take_precedence(self.take_token().1)
        {
            left_exp = if let Some(infix) = self.infix_fn() {
                self.next_token();
                infix(self, &left_exp)?
            } else {
                return Err(Error::InfixNoneErr);
            };
        }

        Ok(left_exp)
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
            add;
            asdf;
        "#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![
            Statement::ExpressionStmt {
                expression: Expression::Ident("foobar".to_string()),
            },
            Statement::ExpressionStmt {
                expression: Expression::Ident("add".to_string()),
            },
            Statement::ExpressionStmt {
                expression: Expression::Ident("asdf".to_string()),
            },
        ];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }

    #[test]
    fn parse_integer() -> error::Result<()> {
        let input = r#"
            5;
            155;
            32415;
        "#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![
            Statement::ExpressionStmt {
                expression: Expression::Integer(5),
            },
            Statement::ExpressionStmt {
                expression: Expression::Integer(155),
            },
            Statement::ExpressionStmt {
                expression: Expression::Integer(32415),
            },
        ];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }

    #[test]
    fn parse_prefix() -> error::Result<()> {
        let input = r#"
            !5;
            -15;
        "#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![
            Statement::ExpressionStmt {
                expression: Expression::Prefix {
                    operator: Token::BANG,
                    right: Box::new(Expression::Integer(5)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Prefix {
                    operator: Token::MINUS,
                    right: Box::new(Expression::Integer(15)),
                },
            },
        ];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }

    #[test]
    fn parse_infix() -> error::Result<()> {
        let input = r#"
            5 + 6;
            5 - 7;
            5 * 8;
            5 / 9;
            5 > 10;
            5 < 11;
            5 == 12;
            5 != 13;
        "#;
        let mut parser = Parser::new(Lexer::new(&input));
        let expected = vec![
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::PLUS,
                    right: Box::new(Expression::Integer(6)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::MINUS,
                    right: Box::new(Expression::Integer(7)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::ASTERISK,
                    right: Box::new(Expression::Integer(8)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::SLASH,
                    right: Box::new(Expression::Integer(9)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::GT,
                    right: Box::new(Expression::Integer(10)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::LT,
                    right: Box::new(Expression::Integer(11)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::EQ,
                    right: Box::new(Expression::Integer(12)),
                },
            },
            Statement::ExpressionStmt {
                expression: Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::NOTEQ,
                    right: Box::new(Expression::Integer(13)),
                },
            },
        ];
        assert_eq!(expected, parser.parse_program()?);
        Ok(())
    }
}
