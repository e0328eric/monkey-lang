pub mod ast;

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

    fn take_token(&self) -> (&Token, &Token, &Token) {
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
        let twopeek_tok = if self.cur_position >= self.l.len() - 2 {
            &Token::EOF
        } else {
            &self.l[self.cur_position + 2]
        };
        (cur_tok, peek_tok, twopeek_tok)
    }

    pub fn new(l: Lexer<'_>) -> Self {
        let l: Vec<Token> = l.collect();
        Self { l, cur_position: 0 }
    }

    pub fn parse_program(&mut self) -> error::Result<Program> {
        let mut program: Program = vec![];
        while self.take_token().0 != &Token::EOF {
            program.push(self.parse_statement()?);
            self.next_token();
        }
        Ok(program)
    }

    fn prefix_fn(&mut self) -> Option<PrefixParseFn> {
        match self.take_token().0 {
            Token::IDENT(_) => Some(Parser::parse_identifier),
            Token::INT(_) => Some(Parser::parse_number),
            Token::IMEGINARY(_) => Some(Parser::parse_number),
            Token::TRUE => Some(Parser::parse_boolean),
            Token::FALSE => Some(Parser::parse_boolean),
            Token::BANG => Some(Parser::parse_prefix_expr),
            Token::MINUS => Some(Parser::parse_prefix_expr),
            Token::LPAREN => Some(Parser::parse_grouped_expr),
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
            _ => None,
        }
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
        expect_peek!(self => Token::RPAREN | Token::RPAREN);

        Ok(exp)
    }

    fn parse_if_expr(&mut self) -> error::Result<Expression> {
        expect_peek!(self => Token::LPAREN | Token::LPAREN);

        self.next_token();
        let condition = Box::new(self.parse_expression(Precedence::LOWEST)?);

        expect_peek!(self => Token::RPAREN | Token::RPAREN);
        expect_peek!(self => Token::LBRACE | Token::LBRACE);

        let consequence = self.parse_block_statement()?;
        let mut alternative: BlockStmt = Vec::new();

        if self.take_token().1 == &Token::ELSE {
            self.next_token();
            expect_peek!(self => Token::LBRACE | Token::LBRACE);
            alternative = self.parse_block_statement()?;
        }

        Ok(Expression::IfExpr {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_function_literal(&mut self) -> error::Result<Expression> {
        expect_peek!(self => Token::LPAREN | Token::LPAREN);
        let parameter = self.parse_function_parameters()?;
        expect_peek!(self => Token::LBRACE | Token::LBRACE);
        let body = self.parse_block_statement()?;

        Ok(Expression::Function { parameter, body })
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

        expect_peek!(self => Token::RPAREN | Token::RPAREN);

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
        let arguments = self.parse_call_arguments()?;
        Ok(Expression::Call {
            function: Box::new(fnt.clone()),
            arguments,
        })
    }

    fn parse_call_arguments(&mut self) -> error::Result<Vec<Expression>> {
        let mut args: Vec<Expression> = Vec::new();

        if self.take_token().1 == &Token::RPAREN {
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

        expect_peek!(self => Token::RPAREN | Token::RPAREN);

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

    fn parse_statement(&mut self) -> error::Result<Statement> {
        match self.take_token().0 {
            Token::LET => self.parse_let_stmt(),
            Token::RETURN => self.parse_return_stmt(),
            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> error::Result<Statement> {
        expect_peek!(self => Token::IDENT(_) | Token::IDENT(String::new()));
        let name = self.take_token().0.unwrap_string()?;
        expect_peek!(self => Token::ASSIGN | Token::ASSIGN);
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error;
    use crate::lexer::Lexer;

    macro_rules! test_parser {
        ($test: ident => $input: expr; $($expected: expr),*) => {
            #[test]
            fn $test() -> error::Result<()> {
                let input = $input;
                let mut parser = Parser::new(Lexer::new(&input));
                let mut expected: Vec<Statement> = Vec::new();
                $(expected.push($expected);)*
                assert_eq!(expected, parser.parse_program()?);
                Ok(())
            }
        };
    }

    test_parser!(
        parse_let => r#"let x = 5;
        let y = 10;
        let foobar = 838383;"#;
        Statement::LetStmt {
            name: "x".to_string(),
            value: Expression::Integer(5),
        },
        Statement::LetStmt {
            name: "y".to_string(),
            value: Expression::Integer(10),
        },
        Statement::LetStmt {
            name: "foobar".to_string(),
            value: Expression::Integer(838383),
        }
    );

    test_parser!(
        parse_return => r#" return 5;
        return 10;
        return 993322;"#;
        Statement::ReturnStmt {
            value: Expression::Integer(5),
        },
        Statement::ReturnStmt {
            value: Expression::Integer(10),
        },
        Statement::ReturnStmt {
            value: Expression::Integer(993322),
        }
    );

    test_parser!(
        parse_ident => r#" foobar;
        add;
        asdf; "#;
        Statement::ExpressionStmt {
            expression: Expression::Ident("foobar".to_string()),
        },
        Statement::ExpressionStmt {
            expression: Expression::Ident("add".to_string()),
        },
        Statement::ExpressionStmt {
            expression: Expression::Ident("asdf".to_string()),
        }
    );

    test_parser!(
        parse_integer => r#" 5;
        155;
        32415;"#;
        Statement::ExpressionStmt {
            expression: Expression::Integer(5),
        },
        Statement::ExpressionStmt {
            expression: Expression::Integer(155),
        },
        Statement::ExpressionStmt {
            expression: Expression::Integer(32415),
        }
    );

    test_parser!(
        parse_complex => r#" 5j;
        1 + 12j;
        532 - 221j;"#;
        Statement::ExpressionStmt {
            expression: Expression::Complex { re: 0, im: 5 },
        },
        Statement::ExpressionStmt {
            expression: Expression::Complex { re: 1, im: 12 },
        },
        Statement::ExpressionStmt {
            expression: Expression::Complex { re: 532, im: -221 },
        }
    );

    test_parser!(
        parse_prefix => r#"!5;
            -15;"#;
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
        }
    );

    test_parser!(
        parse_infix => r#"5 + 6;
        5 - 7;
        5 * 8;
        5 / 9;
        5 > 10;
        5 < 11;
        5 == 12;
        5 != 13; "#;
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
        }
    );

    test_parser!(
        parse_complex_infix => r#"3 - - 2;
        2 + 3j + 5 - 4j;
        3 + 4 * - 5 == 3 * 1 + -4 / 5;"#;
        Statement::ExpressionStmt {
            expression: Expression::Infix {
                left: Box::new(Expression::Integer(3)),
                operator: Token::MINUS,
                right: Box::new(Expression::Prefix {
                    operator: Token::MINUS,
                    right: Box::new(Expression::Integer(2)),
                }),
            },
        },
        Statement::ExpressionStmt {
            expression: Expression::Infix {
                left: Box::new(Expression::Complex {
                    re: 2,
                    im: 3,
                }),
                operator: Token::PLUS,
                right: Box::new(Expression::Complex {
                    re: 5,
                    im: -4,
                }),
            }
        },
        Statement::ExpressionStmt {
            expression: Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(3)),
                    operator: Token::PLUS,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer(4)),
                        operator: Token::ASTERISK,
                        right: Box::new(Expression::Prefix {
                            operator: Token::MINUS,
                            right: Box::new(Expression::Integer(5)),
                        }),
                    }),
                }),
                operator: Token::EQ,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer(3)),
                        operator: Token::ASTERISK,
                        right: Box::new(Expression::Integer(1)),
                    }),
                    operator: Token::PLUS,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Prefix {
                            operator: Token::MINUS,
                            right: Box::new(Expression::Integer(4)),
                        }),
                        operator: Token::SLASH,
                        right: Box::new(Expression::Integer(5)),
                    }),
                }),
            },
        }
    );

    test_parser!(
        parse_bools => r#"true;
        false;
        let foobar = true;
        let barfoo = false;
        3 > 5 == false;
        !true;"#;
        Statement::ExpressionStmt {
            expression: Expression::Boolean(true),
        },
        Statement::ExpressionStmt {
            expression: Expression::Boolean(false),
        },
        Statement::LetStmt {
            name: "foobar".to_string(),
            value: Expression::Boolean(true),
        },
        Statement::LetStmt {
            name: "barfoo".to_string(),
            value: Expression::Boolean(false),
        },
        Statement::ExpressionStmt {
            expression: Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(3)),
                    operator: Token::GT,
                    right: Box::new(Expression::Integer(5))
                }),
                operator: Token::EQ,
                right: Box::new(Expression::Boolean(false))
            }
        },
        Statement::ExpressionStmt {
            expression: Expression::Prefix {
                operator: Token::BANG,
                right: Box::new(Expression::Boolean(true))
            }
        }
    );

    test_parser!(
        parse_grouped => r#"-(5+6);
        (2 + 3) * 4;"#;
        Statement::ExpressionStmt {
            expression: Expression::Prefix {
                operator: Token::MINUS,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::PLUS,
                    right: Box::new(Expression::Integer(6)),
                })
            }
        },
        Statement::ExpressionStmt {
            expression: Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(2)),
                    operator: Token::PLUS,
                    right: Box::new(Expression::Integer(3)),
                }),
                operator: Token::ASTERISK,
                right: Box::new(Expression::Integer(4)),
            }
        }
    );

    test_parser!(
        parse_if_expr => r#"
        if (x < y) { x };
        "#;
        Statement::ExpressionStmt {
            expression: Expression::IfExpr {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Ident("x".to_string())),
                    operator: Token::LT,
                    right: Box::new(Expression::Ident("y".to_string())),
                }),
                consequence: vec![Statement::ExpressionStmt { expression: Expression::Ident("x".to_string()) }],
                alternative: vec![]
            }
        }
    );

    test_parser!(
        parse_if_else_expr => r#"
        if(foo!=bar){bar}else{foo};
        "#;
        Statement::ExpressionStmt {
            expression: Expression::IfExpr {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Ident("foo".to_string())),
                    operator: Token::NOTEQ,
                    right: Box::new(Expression::Ident("bar".to_string())),
                }),
                consequence: vec![Statement::ExpressionStmt { expression: Expression::Ident("bar".to_string()) }],
                alternative: vec![Statement::ExpressionStmt { expression: Expression::Ident("foo".to_string()) }],
            }
        }
    );

    test_parser!(
        parse_function_literal => r#"
        fn(x, y) { x + y; };
        fn() {};
        fn(x, y, z) {};
        "#;
        Statement::ExpressionStmt {
            expression: Expression::Function {
                parameter: vec!["x".to_string(), "y".to_string()],
                body: vec![
                    Statement::ExpressionStmt {
                    expression: Expression::Infix {
                    left: Box::new(Expression::Ident("x".to_string())),
                    operator: Token::PLUS,
                    right: Box::new(Expression::Ident("y".to_string())),
                }}]
            }
        },
        Statement::ExpressionStmt {
            expression: Expression::Function {
                parameter: vec![],
                body: vec![]
            }
        },
        Statement::ExpressionStmt {
            expression: Expression::Function {
                parameter: vec!["x".to_string(), "y".to_string(), "z".to_string()],
                body: vec![]
            }
        }
    );

    test_parser!(
        parse_call_expr => r#"
        add(1, 2 * 3, 4 + 5);
        "#;
        Statement::ExpressionStmt {
            expression: Expression::Call {
                function: Box::new(Expression::Ident("add".to_string())),
                arguments: vec![
                    Expression::Integer(1),
                    Expression::Infix {
                        left: Box::new(Expression::Integer(2)),
                        operator: Token::ASTERISK,
                        right: Box::new(Expression::Integer(3))
                    },
                    Expression::Infix {
                        left: Box::new(Expression::Integer(4)),
                        operator: Token::PLUS,
                        right: Box::new(Expression::Integer(5))
                    }
                ]
            }
        }
    );
}
