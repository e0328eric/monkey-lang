use super::*;
use crate::error;
use crate::lexer::Lexer;

macro_rules! test_parser {
        ($test: ident => $input: expr; $($expected: expr),*) => {
            #[test]
            fn $test() -> error::Result<()> {
                let input = $input;
                let mut parser = Parser::new(Lexer::new(&input));
                let expected: Vec<Statement> = vec![$($expected,)*];
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
    parse_complex => r#"5i;
        1 + 12i;
        532 - 221i;"#;
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
        2 + 3i + 5 - 4i;
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
            alternative: None
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
            alternative: Some(vec![Statement::ExpressionStmt { expression: Expression::Ident("foo".to_string()) }]),
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
            parameters: vec!["x".to_string(), "y".to_string()],
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
            parameters: vec![],
            body: vec![]
        }
    },
    Statement::ExpressionStmt {
        expression: Expression::Function {
            parameters: vec!["x".to_string(), "y".to_string(), "z".to_string()],
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

test_parser!(
    parse_array => r#"
            [1, 2*2, 3+3];
            a * [1,2,3,4][b*c] * d;
            "#;
    Statement::ExpressionStmt {
        expression: Expression::Array(vec![
            Expression::Integer(1),
            Expression::Infix {
                left: Box::new(Expression::Integer(2)),
                operator: Token::ASTERISK,
                right: Box::new(Expression::Integer(2))
            },
            Expression::Infix {
                left: Box::new(Expression::Integer(3)),
                operator: Token::PLUS,
                right: Box::new(Expression::Integer(3))
            },
        ]),
    },
    Statement::ExpressionStmt {
        expression: Expression::Infix {
            left: Box::new(Expression::Infix {
                left: Box::new(Expression::Ident("a".to_string())),
                operator: Token::ASTERISK,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Array(vec![
                        Expression::Integer(1),
                        Expression::Integer(2),
                        Expression::Integer(3),
                        Expression::Integer(4),
                    ])),
                    operator: Token::LBRACKET,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Ident("b".to_string())),
                        operator: Token::ASTERISK,
                        right: Box::new(Expression::Ident("c".to_string())),
                    })
                })
            }),
            operator: Token::ASTERISK,
            right: Box::new(Expression::Ident("d".to_string())),
        }
    }
);

test_parser!(
    parse_hash => r#"
            {"one": 1, "two": 2, "three": 3};
            "#;
    Statement::ExpressionStmt {
        expression: Expression::Hash {
            key: vec![
                Expression::String("one".to_string()),
                Expression::String("two".to_string()),
                Expression::String("three".to_string()),
            ],
            value: vec![
                Expression::Integer(1),
                Expression::Integer(2),
                Expression::Integer(3),
            ]
        }
    }
);
