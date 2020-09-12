use crate::lexer::token::Token;
use crate::object::Object;
use crate::parser::ast::*;

// To make unique objects
const TRUE: Object = Object::Boolean { value: true };
const FALSE: Object = Object::Boolean { value: false };
const NULL: Object = Object::Null;

pub fn eval(node: Statement) -> Object {
    if let Statement::ExpressionStmt { expression } = node {
        match expression {
            Expression::Integer(value) => Object::Integer { value },
            Expression::Boolean(value) => {
                if value {
                    TRUE
                } else {
                    FALSE
                }
            }
            Expression::Prefix { operator, right } => {
                eval_prefix_expr(operator, eval(right.into()))
            }

            Expression::Infix {
                left,
                operator,
                right,
            } => eval_infix_expr(operator, eval(left.into()), eval(right.into())),
            _ => NULL,
        }
    } else {
        NULL
    }
}

fn eval_prefix_expr(operator: Token, right: Object) -> Object {
    match operator {
        Token::BANG => eval_bang_operator_expr(right),
        Token::MINUS => eval_minus_operator_expr(right),
        _ => NULL,
    }
}

fn eval_infix_expr(operator: Token, left: Object, right: Object) -> Object {
    match (&left, &right) {
        (&Object::Integer { value: left }, &Object::Integer { value: right }) => {
            eval_integer_infix_expr(operator, left, right)
        }
        _ => match operator {
            Token::EQ => {
                if left == right {
                    TRUE
                } else {
                    FALSE
                }
            }
            Token::NOTEQ => {
                if left != right {
                    TRUE
                } else {
                    FALSE
                }
            }
            _ => NULL,
        },
    }
}

fn eval_bang_operator_expr(right: Object) -> Object {
    match right {
        TRUE => FALSE,
        FALSE => TRUE,
        NULL => TRUE, // This means that NULL is falsty
        _ => FALSE,   // and the defalut is truthy
    }
}

fn eval_minus_operator_expr(right: Object) -> Object {
    if let Object::Integer { value } = right {
        Object::Integer { value: -value }
    } else {
        NULL
    }
}

fn eval_integer_infix_expr(operator: Token, left: i64, right: i64) -> Object {
    match operator {
        Token::PLUS => Object::Integer {
            value: left + right,
        },
        Token::MINUS => Object::Integer {
            value: left - right,
        },
        Token::ASTERISK => Object::Integer {
            value: left * right,
        },
        Token::SLASH => Object::Integer {
            value: left / right,
        },
        Token::LT => Object::Boolean {
            value: left < right,
        },
        Token::GT => Object::Boolean {
            value: left > right,
        },
        Token::EQ => Object::Boolean {
            value: left == right,
        },
        Token::NOTEQ => Object::Boolean {
            value: left != right,
        },
        _ => NULL,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn eval_integers() -> error::Result<()> {
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            5; 10;
            -5; -10;
            5 + 5 + 5 + 5 - 10;
            2 * 2 * 2 * 2 * 2;
            -50 + 100 - 50;
            5 * 2 + 10;
            5 + 2 * 10;
            20 + 2 * -10;
            50 / 2 * 2 + 10;
            2 * (5 + 10);
            3 * 3 * 3 + 10;
            3 * (3 * 3) + 10;
            (5 + 10 * 2 + 15 / 3) * 2 + -10;
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(eval)
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer { value: 5 },
                Object::Integer { value: 10 },
                Object::Integer { value: -5 },
                Object::Integer { value: -10 },
                Object::Integer { value: 10 },
                Object::Integer { value: 32 },
                Object::Integer { value: 0 },
                Object::Integer { value: 20 },
                Object::Integer { value: 25 },
                Object::Integer { value: 0 },
                Object::Integer { value: 60 },
                Object::Integer { value: 30 },
                Object::Integer { value: 37 },
                Object::Integer { value: 37 },
                Object::Integer { value: 50 },
            ]
        );
        Ok(())
    }
    #[test]
    fn eval_boolean() -> error::Result<()> {
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            false;
            true;
            1 < 2;
            1 > 2;
            1 < 1;
            1 > 1;
            1 == 1;
            1 != 1;
            1 == 2;
            1 != 2;
            true == true;
            true == false;
            false == true;
            true != false;
            false != true;
            false != false;
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(eval)
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_mixed() -> error::Result<()> {
        let input: Vec<_> = Parser::new(Lexer::new("7; true; 15; false; false; 324;"))
            .parse_program()?
            .into_iter()
            .map(eval)
            .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer { value: 7 },
                Object::Boolean { value: true },
                Object::Integer { value: 15 },
                Object::Boolean { value: false },
                Object::Boolean { value: false },
                Object::Integer { value: 324 },
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_bang_operator() -> error::Result<()> {
        let input: Vec<_> = Parser::new(Lexer::new("!true; !false !5; !!true; !!false; !!5;"))
            .parse_program()?
            .into_iter()
            .map(eval)
            .collect();
        assert_eq!(
            input,
            vec![
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
                Object::Boolean { value: false },
                Object::Boolean { value: true },
            ]
        );
        Ok(())
    }
}
