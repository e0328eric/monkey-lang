#[macro_use]
mod macros;
pub mod ast;
#[cfg(test)]
mod parser_test;

use crate::error;
use crate::lexer::token::Token;
use crate::lexer::Lexer;
use crate::parser::ast::{BlockStmt, Expression, Precedence, Program, Statement};

type Error = error::MonkeyErr;
type PrefixParseFn = fn(&mut Parser) -> error::Result<Expression>;
type InfixParseFn = fn(&mut Parser, &Expression) -> error::Result<Expression>;

pub struct Parser {
    l: Vec<Token>,
    cur_position: usize,
}

impl Parser {
    fn next_token(&mut self) {
        self.cur_position += 1;
    }

    fn take_token(&self) -> (&Token, &Token, &Token) {
        check_position!(cur_tok := self, 0);
        check_position!(peek_tok := self, 1);
        check_position!(twopeek_tok := self, 2);
        (cur_tok, peek_tok, twopeek_tok)
    }

    pub fn new(l: Lexer<'_>) -> Self {
        let l: Vec<Token> = l.collect();
        Self { l, cur_position: 0 }
    }

    pub fn parse_program(&mut self) -> error::Result<Program> {
        let mut program = Program::new(vec![]);
        while self.take_token().0 != &Token::EOF {
            program.push(self.parse_statement()?);
            self.next_token();
        }
        Ok(program)
    }

    fn prefix_fn(&mut self) -> Option<PrefixParseFn> {
        match self.take_token().0 {
            Token::IDENT(_) => Some(Parser::parse_identifier),
            Token::STRING(_) => Some(Parser::parse_string),
            Token::INT(_) => Some(Parser::parse_number),
            Token::IMEGINARY(_) => Some(Parser::parse_number),
            Token::TRUE => Some(Parser::parse_boolean),
            Token::FALSE => Some(Parser::parse_boolean),
            Token::BANG => Some(Parser::parse_prefix_expr),
            Token::MINUS => Some(Parser::parse_prefix_expr),
            Token::LPAREN => Some(Parser::parse_grouped_expr),
            Token::LBRACE => Some(Parser::parse_hash_expr),
            Token::LBRACKET => Some(Parser::parse_array_expr),
            Token::IF => Some(Parser::parse_if_expr),
            Token::FUNCTION => Some(Parser::parse_function_literal),
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
            Token::POWER => Some(Parser::parse_infix_expr),
            Token::LPAREN => Some(Parser::parse_call_expr),
            Token::LBRACKET => Some(Parser::parse_index_expr),
            _ => None,
        }
    }

    fn parse_statement(&mut self) -> error::Result<Statement> {
        match self.take_token().0 {
            Token::LET => self.parse_let_stmt(),
            Token::RETURN => self.parse_return_stmt(),
            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> error::Result<Statement> {
        expect_peek!(self => Token::IDENT(String::new()));
        let name = self.take_token().0.unwrap_string()?;
        expect_peek!(self => Token::ASSIGN);
        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST)?;

        if self.take_token().0 != &Token::SEMICOLON {
            self.next_token();
        }

        Ok(Statement::LetStmt { name, value })
    }

    fn parse_return_stmt(&mut self) -> error::Result<Statement> {
        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST)?;

        if self.take_token().0 != &Token::SEMICOLON {
            self.next_token();
        }
        Ok(Statement::ReturnStmt { value })
    }

    fn parse_expression_stmt(&mut self) -> error::Result<Statement> {
        let expression = self.parse_expression(Precedence::LOWEST)?;
        if self.take_token().1 == &Token::SEMICOLON {
            self.next_token();
        }
        Ok(Statement::ExpressionStmt { expression })
    }

    fn parse_expression(&mut self, prece: Precedence) -> error::Result<Expression> {
        let mut left_exp = if let Some(prefix) = self.prefix_fn() {
            prefix(self)?
        } else {
            return Err(Error::PrefixParseNoneErr {
                got: self.take_token().0.clone(),
            });
        };

        while self.take_token().1 != &Token::SEMICOLON
            && prece < Precedence::take_precedence(self.take_token().1)
        {
            left_exp = if let Some(infix) = self.infix_fn() {
                self.next_token();
                infix(self, &left_exp)?
            } else {
                return Err(Error::InfixParseNoneErr {
                    got: self.take_token().1.clone(),
                });
            };
        }

        Ok(left_exp)
    }

    fn parse_identifier(&mut self) -> error::Result<Expression> {
        if let Token::IDENT(i) = self.take_token().0 {
            Ok(Expression::Ident(i.to_string()))
        } else {
            Err(Error::ParseExprErr {
                expected: "identifier".to_string(),
                got: self.take_token().0.clone(),
            })
        }
    }

    fn parse_string(&mut self) -> error::Result<Expression> {
        if let Token::STRING(s) = self.take_token().0 {
            Ok(Expression::String(s.to_string()))
        } else {
            Err(Error::ParseExprErr {
                expected: "string".to_string(),
                got: self.take_token().0.clone(),
            })
        }
    }

    fn parse_number(&mut self) -> error::Result<Expression> {
        let (expr, move_num) = match self.take_token().0 {
            Token::IMEGINARY(i) => (Expression::Complex { re: 0, im: *i }, false),
            Token::INT(n) => {
                if let Token::IMEGINARY(i) = self.take_token().2 {
                    if self.take_token().1 == &Token::PLUS {
                        (Expression::Complex { re: *n, im: *i }, true)
                    } else if self.take_token().1 == &Token::MINUS {
                        (Expression::Complex { re: *n, im: -*i }, true)
                    } else {
                        (Expression::Integer(*n), false)
                    }
                } else {
                    (Expression::Integer(*n), false)
                }
            }
            _ => {
                return Err(Error::ParseExprErr {
                    expected: "number".to_string(),
                    got: self.take_token().0.clone(),
                })
            }
        };

        if move_num {
            self.next_token();
            self.next_token();
        }
        Ok(expr)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn parse_boolean(&mut self) -> error::Result<Expression> {
        Ok(Expression::Boolean(self.take_token().0 == &Token::TRUE))
    }

    fn parse_prefix_expr(&mut self) -> error::Result<Expression> {
        let operator = self.take_token().0.clone();
        self.next_token();

        let right = Box::new(self.parse_expression(Precedence::PREFIX)?);
        Ok(Expression::Prefix { operator, right })
    }

    fn parse_grouped_expr(&mut self) -> error::Result<Expression> {
        self.next_token();

        let exp = self.parse_expression(Precedence::LOWEST)?;
        expect_peek!(self => Token::RPAREN);

        Ok(exp)
    }

    fn parse_array_expr(&mut self) -> error::Result<Expression> {
        Ok(Expression::Array(self.parse_expr_list(Token::RBRACKET)?))
    }

    fn parse_hash_expr(&mut self) -> error::Result<Expression> {
        let mut key = Vec::<Expression>::new();
        let mut value = Vec::<Expression>::new();

        while self.take_token().1 != &Token::RBRACE {
            self.next_token();
            key.push(self.parse_expression(Precedence::LOWEST)?);
            expect_peek!(self => Token::COLON);

            self.next_token();
            value.push(self.parse_expression(Precedence::LOWEST)?);

            if self.take_token().1 != &Token::RBRACE {
                expect_peek!(self => Token::COMMA);
            }
        }

        expect_peek!(self => Token::RBRACE);

        Ok(Expression::Hash { key, value })
    }

    fn parse_index_expr(&mut self, left: &Expression) -> error::Result<Expression> {
        self.next_token();
        let right = Box::new(self.parse_expression(Precedence::LOWEST)?);
        expect_peek!(self => Token::RBRACKET);

        Ok(Expression::Infix {
            left: Box::new(left.clone()),
            operator: Token::LBRACKET,
            right,
        })
    }

    fn parse_if_expr(&mut self) -> error::Result<Expression> {
        expect_peek!(self => Token::LPAREN);

        self.next_token();
        let condition = Box::new(self.parse_expression(Precedence::LOWEST)?);

        expect_peek!(self => Token::RPAREN);
        expect_peek!(self => Token::LBRACE);

        let consequence = self.parse_block_statement()?;
        let mut alternative: Option<BlockStmt> = None;

        if self.take_token().1 == &Token::ELSE {
            self.next_token();
            expect_peek!(self => Token::LBRACE);
            alternative = Some(self.parse_block_statement()?);
        }

        Ok(Expression::IfExpr {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_function_literal(&mut self) -> error::Result<Expression> {
        expect_peek!(self => Token::LPAREN);
        let parameters = self.parse_function_parameters()?;
        expect_peek!(self => Token::LBRACE);
        let body = self.parse_block_statement()?;

        Ok(Expression::Function { parameters, body })
    }

    fn parse_function_parameters(&mut self) -> error::Result<Vec<String>> {
        let mut identifiers: Vec<_> = Vec::new();
        if self.take_token().1 == &Token::RPAREN {
            self.next_token();
            return Ok(identifiers);
        }

        self.next_token();
        identifiers.push(self.take_token().0.unwrap_string()?);

        while self.take_token().1 == &Token::COMMA {
            self.next_token();
            self.next_token();
            identifiers.push(self.take_token().0.unwrap_string()?);
        }

        expect_peek!(self => Token::RPAREN);

        Ok(identifiers)
    }

    fn parse_block_statement(&mut self) -> error::Result<BlockStmt> {
        let mut stmts: BlockStmt = Vec::new();
        self.next_token();

        while self.take_token().0 != &Token::RBRACE && self.take_token().0 != &Token::EOF {
            stmts.push(self.parse_statement()?);
            self.next_token();
        }

        Ok(stmts)
    }

    fn parse_call_expr(&mut self, fnt: &Expression) -> error::Result<Expression> {
        let arguments = self.parse_expr_list(Token::RPAREN)?;
        Ok(Expression::Call {
            function: Box::new(fnt.clone()),
            arguments,
        })
    }

    fn parse_expr_list(&mut self, end: Token) -> error::Result<Vec<Expression>> {
        let mut args: Vec<Expression> = Vec::new();

        if self.take_token().1 == &end {
            self.next_token();
            return Ok(args);
        }
        self.next_token();
        args.push(self.parse_expression(Precedence::LOWEST)?);

        while self.take_token().1 == &Token::COMMA {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::LOWEST)?);
        }

        expect_peek!(self => end);

        Ok(args)
    }

    fn parse_infix_expr(&mut self, left: &Expression) -> error::Result<Expression> {
        let operator = self.take_token().0.clone();
        let precedence = Precedence::take_precedence(self.take_token().0);
        self.next_token();

        let right = Box::new(self.parse_expression(precedence)?);
        Ok(Expression::Infix {
            left: Box::new(left.clone()),
            operator,
            right,
        })
    }
}
