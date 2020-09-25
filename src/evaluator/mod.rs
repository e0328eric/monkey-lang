use crate::error;
use crate::lexer::token::Token;
use crate::object::{Builtin, Environment, Object, FALSE, NULL, TRUE};
use crate::parser::ast::*;

type Error = crate::error::MonkeyErr;

pub fn eval_program(stmts: Vec<Statement>, env: &mut Environment) -> error::Result<Object> {
    let mut result: Object = NULL;
    for statement in stmts {
        result = eval(statement, env)?;
        if let Object::ReturnValue(value) = result {
            return Ok(*value);
        }
    }
    Ok(result)
}

pub fn eval(node: Statement, env: &mut Environment) -> error::Result<Object> {
    match node {
        Statement::LetStmt { name, value } => {
            let val = eval(value.into(), env)?;
            Ok(env.set(name, val))
        }
        Statement::ReturnStmt { value } => {
            Ok(Object::ReturnValue(Box::new(eval(value.into(), env)?)))
        }
        Statement::ExpressionStmt { expression } => match expression {
            Expression::String(value) => Ok(Object::String(value)),
            Expression::Integer(value) => Ok(Object::Integer(value)),
            Expression::Complex { re, im } => Ok(Object::Complex(re, im)),
            Expression::Ident(value) => eval_identifier(value, env),
            Expression::Array(exprs) => Ok(Object::Array(eval_exprs(exprs, env)?)),
            Expression::Boolean(value) => {
                if value {
                    Ok(TRUE)
                } else {
                    Ok(FALSE)
                }
            }
            Expression::Prefix { operator, right } => {
                eval_prefix_expr(operator, eval(right.into(), env)?)
            }

            Expression::Infix {
                left,
                operator,
                right,
            } => {
                if operator == Token::LBRACKET {
                    eval_index_eval(eval(left.into(), env)?, eval(right.into(), env)?)
                } else {
                    eval_infix_expr(operator, eval(left.into(), env)?, eval(right.into(), env)?)
                }
            }
            Expression::IfExpr {
                condition,
                consequence,
                alternative,
            } => eval_if_expr(eval(condition.into(), env)?, consequence, alternative, env),
            Expression::Function { parameters, body } => Ok(Object::Function { parameters, body }),
            Expression::Call {
                function,
                arguments,
            } => {
                let fnt = eval(function.into(), env)?;
                let args = eval_exprs(arguments, env)?;
                apply_function(fnt, args, env)
            }
        },
    }
}

fn apply_function(fnt: Object, args: Vec<Object>, env: &mut Environment) -> error::Result<Object> {
    match fnt {
        Object::Function { parameters, body } => {
            let mut extended_env = extended_function_env(parameters, env, args)?;
            let evaluated = eval_stmts(body, &mut extended_env)?;
            if let Object::ReturnValue(obj) = evaluated {
                Ok(*obj)
            } else {
                Ok(evaluated)
            }
        }
        Object::BuiltinFnt(builtfnt) => builtfnt.apply(args),
        _ => Err(Error::EvalNotFunction),
    }
}

fn extended_function_env(
    parameters: Vec<String>,
    envr: &Environment,
    args: Vec<Object>,
) -> error::Result<Environment> {
    let mut env = envr.new_enclosed_env();
    for (para, arg) in parameters.into_iter().zip(args.into_iter()) {
        env.set(para, arg);
    }
    Ok(env)
}

fn eval_exprs(exprs: Vec<Expression>, env: &mut Environment) -> error::Result<Vec<Object>> {
    let mut result = Vec::new();
    for expr in exprs {
        result.push(eval(expr.into(), env)?);
    }
    Ok(result)
}

fn eval_identifier(ident: String, env: &mut Environment) -> error::Result<Object> {
    let value = env.get(&ident);
    if let Some(val) = value {
        Ok(val)
    } else if ident != Builtin::ConvertErr {
        Ok(Object::BuiltinFnt(ident.into()))
    } else {
        Err(Error::EvalIdentNotFound { name_got: ident })
    }
}

fn eval_stmts(block: BlockStmt, env: &mut Environment) -> error::Result<Object> {
    let mut result: Object = NULL;
    for statement in block {
        result = eval(statement, env)?;
        if let Object::ReturnValue { .. } = result {
            return Ok(result);
        }
    }
    Ok(result)
}

fn eval_index_eval(left: Object, index: Object) -> error::Result<Object> {
    if let (Object::Array(lst), Object::Integer(n)) = (&left, index) {
        if n < 0 {
            Ok(NULL)
        } else {
            let n = n as usize;
            if n > lst.len() {
                Ok(NULL)
            } else {
                Ok(lst[n].clone())
            }
        }
    } else {
        Err(Error::EvalIndexOpErr { got: left.clone() })
    }
}

fn eval_prefix_expr(operator: Token, right: Object) -> error::Result<Object> {
    match operator {
        Token::BANG => eval_bang_operator_expr(right),
        Token::MINUS => eval_minus_operator_expr(right),
        _ => Err(Error::EvalUnknownPrefix { operator, right }),
    }
}

fn eval_infix_expr(operator: Token, left: Object, right: Object) -> error::Result<Object> {
    if left.to_complex().is_some() && right.to_complex().is_some() {
        eval_num_infix_expr(operator, left, right)
    } else {
        match (&left, &right) {
            (Object::String(s1), Object::String(s2)) => {
                if let Token::PLUS = operator {
                    Ok(Object::String(s1.to_string() + s2))
                } else {
                    Err(Error::EvalUnknownInfix {
                        left,
                        operator,
                        right,
                    })
                }
            }
            _ if Object::is_same_type(&left, &right) => match operator {
                Token::EQ => {
                    if left == right {
                        Ok(TRUE)
                    } else {
                        Ok(FALSE)
                    }
                }
                Token::NOTEQ => {
                    if left != right {
                        Ok(TRUE)
                    } else {
                        Ok(FALSE)
                    }
                }
                _ => Err(Error::EvalUnknownInfix {
                    left,
                    operator,
                    right,
                }),
            },
            _ => Err(Error::EvalTypeMismatch {
                left,
                operator,
                right,
            }),
        }
    }
}

fn eval_if_expr(
    cond: Object,
    consq: BlockStmt,
    alter: BlockStmt,
    env: &mut Environment,
) -> error::Result<Object> {
    if is_truthy(&cond) {
        eval_stmts(consq, env)
    } else if !alter.is_empty() {
        eval_stmts(alter, env)
    } else {
        Ok(NULL)
    }
}

fn eval_bang_operator_expr(right: Object) -> error::Result<Object> {
    match right {
        TRUE => Ok(FALSE),
        FALSE => Ok(TRUE),
        NULL => Ok(TRUE), // This means that NULL is falsty
        _ => Ok(FALSE),   // and the defalut is truthy
    }
}

fn eval_minus_operator_expr(right: Object) -> error::Result<Object> {
    match right {
        Object::Integer(value) => Ok(Object::Integer(-value)),
        Object::Complex(re, im) => Ok(Object::Complex(-re, -im)),
        _ => Err(Error::EvalUnknownPrefix {
            operator: Token::MINUS,
            right,
        }),
    }
}

fn eval_num_infix_expr(operator: Token, lf: Object, rt: Object) -> error::Result<Object> {
    use crate::object::Object::Complex;
    // This function called only when both option are some.
    // So unwraping these does not cause panic.
    match (lf.to_complex().unwrap(), rt.to_complex().unwrap()) {
        (Complex(lf, 0), Complex(rt, 0)) => eval_integer_infix_expr(operator, lf, rt),
        (Complex(lf_re, lf_im), Complex(rt_re, rt_im)) => {
            eval_complex_infix_expr(operator, lf_re, lf_im, rt_re, rt_im)
        }
        _ => Err(Error::EvalUnknownInfix {
            left: lf,
            operator,
            right: rt,
        }),
    }
}
fn eval_complex_infix_expr(
    operator: Token,
    lf_re: i64,
    lf_im: i64,
    rt_re: i64,
    rt_im: i64,
) -> error::Result<Object> {
    match operator {
        Token::PLUS => Ok(Object::Complex(lf_re + rt_re, lf_im + rt_im)),
        Token::MINUS => Ok(Object::Complex(lf_re - rt_re, lf_im - rt_im)),
        Token::ASTERISK => Ok(Object::Complex(
            lf_re * rt_re - lf_im * rt_im,
            lf_re * rt_im + lf_im * rt_re,
        )),
        Token::EQ => Ok(Object::Boolean(lf_re == rt_re && lf_im == rt_im)),
        Token::NOTEQ => Ok(Object::Boolean(lf_re != rt_re || lf_im != rt_im)),
        // Division, power operation and ordering are not implemented.
        _ => Err(Error::EvalUnknownInfix {
            left: Object::Complex(lf_re, lf_im),
            operator,
            right: Object::Complex(rt_re, rt_im),
        }),
    }
}

fn eval_integer_infix_expr(operator: Token, lf: i64, rt: i64) -> error::Result<Object> {
    match operator {
        Token::PLUS => Ok(Object::Integer(lf + rt)),
        Token::MINUS => Ok(Object::Integer(lf - rt)),
        Token::ASTERISK => Ok(Object::Integer(lf * rt)),
        Token::SLASH => Ok(Object::Integer(lf / rt)),
        Token::POWER => {
            if rt >= 0 {
                Ok(Object::Integer(lf.pow(rt as u32)))
            } else {
                Err(Error::EvalPowErr)
            }
        }
        Token::LT => Ok(Object::Boolean(lf < rt)),
        Token::GT => Ok(Object::Boolean(lf > rt)),
        Token::EQ => Ok(Object::Boolean(lf == rt)),
        Token::NOTEQ => Ok(Object::Boolean(lf != rt)),
        _ => Err(Error::EvalUnknownInfix {
            left: Object::Integer(lf),
            operator,
            right: Object::Integer(rt),
        }),
    }
}

fn is_truthy(obj: &Object) -> bool {
    match *obj {
        NULL | FALSE => false,
        _ => true,
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
        let mut env = Environment::new();
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
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer(5),
                Object::Integer(10),
                Object::Integer(-5),
                Object::Integer(-10),
                Object::Integer(10),
                Object::Integer(32),
                Object::Integer(0),
                Object::Integer(20),
                Object::Integer(25),
                Object::Integer(0),
                Object::Integer(60),
                Object::Integer(30),
                Object::Integer(37),
                Object::Integer(37),
                Object::Integer(50),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_complex() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            5i; 10i;
            -5i; -10i;
            1 + 4i; 1 - 4i;
            - 1 + 4i; - 1 - 4i;
            (-1) + 4i; (-1) - 4i;
            (-1) + 4i + (-1) - 4i;
            (-1) + 4i - (-1) - 4i;
            ((-1) + 4i) - ((-1) - 4i);
            (-1) + 4i * (-1) - 4i;
            ((-1) + 4i) * ((-1) - 4i);
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Complex(0, 5),
                Object::Complex(0, 10),
                Object::Complex(0, -5),
                Object::Complex(0, -10),
                Object::Complex(1, 4),
                Object::Complex(1, -4),
                Object::Complex(-1, -4),
                Object::Complex(-1, 4),
                Object::Complex(-1, 4),
                Object::Complex(-1, -4),
                Object::Complex(-2, 0),
                Object::Complex(0, 0),
                Object::Complex(0, 8),
                Object::Complex(-1, -8),
                Object::Complex(17, 0),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_boolean() -> error::Result<()> {
        let mut env = Environment::new();
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
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Boolean(false),
                Object::Boolean(true),
                Object::Boolean(true),
                Object::Boolean(false),
                Object::Boolean(false),
                Object::Boolean(false),
                Object::Boolean(true),
                Object::Boolean(false),
                Object::Boolean(false),
                Object::Boolean(true),
                Object::Boolean(true),
                Object::Boolean(false),
                Object::Boolean(false),
                Object::Boolean(true),
                Object::Boolean(true),
                Object::Boolean(false),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_mixed() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new("7; true; 15; false; false; 324;"))
            .parse_program()?
            .into_iter()
            .map(|x| eval(x, &mut env).unwrap())
            .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer(7),
                Object::Boolean(true),
                Object::Integer(15),
                Object::Boolean(false),
                Object::Boolean(false),
                Object::Integer(324),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_bang_operator() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new("!true; !false !5; !!true; !!false; !!5;"))
            .parse_program()?
            .into_iter()
            .map(|x| eval(x, &mut env).unwrap())
            .collect();
        assert_eq!(
            input,
            vec![
                Object::Boolean(false),
                Object::Boolean(true),
                Object::Boolean(false),
                Object::Boolean(true),
                Object::Boolean(false),
                Object::Boolean(true),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_if_else_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            if (true) { 10 };
            if (false) { 10 };
            if (1) { 10 };
            if (1 < 2) { 10 };
            if (1 > 2) { 10 };
            if (1 > 2) { 10 } else { 20 };
            if (1 < 2) { 10 } else { 20 };
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::Integer(10),
                NULL,
                Object::Integer(10),
                Object::Integer(10),
                NULL,
                Object::Integer(20),
                Object::Integer(10),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_return_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input = eval_program(
            Parser::new(Lexer::new(
                r#"
            9; return 2 * 5; 8;
            "#,
            ))
            .parse_program()?,
            &mut env,
        )
        .unwrap();
        assert_eq!(input, Object::Integer(10));
        Ok(())
    }

    #[test]
    fn eval_nested_block_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input = eval_program(
            Parser::new(Lexer::new(
                r#"
            if (10 > 1) {
                if (12 > 2) {
                    return 10;
                }

                return 1;
            }
            "#,
            ))
            .parse_program()?,
            &mut env,
        )
        .unwrap();
        assert_eq!(input, Object::Integer(10));
        Ok(())
    }

    #[test]
    fn eval_let_expr() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            let a = 5; a;
            let a = 5 * 5; a;
            let a = 5; let b = a; b;
            let a = 5; let b = a; let c = a + b + 5; c;
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::DeclareVariable,
                Object::Integer(5),
                Object::DeclareVariable,
                Object::Integer(25),
                Object::DeclareVariable,
                Object::DeclareVariable,
                Object::Integer(5),
                Object::DeclareVariable,
                Object::DeclareVariable,
                Object::DeclareVariable,
                Object::Integer(15),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_function_object() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            fn(x) { x + 2; };
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![Object::Function {
                parameters: vec!["x".to_string()],
                body: vec![Statement::ExpressionStmt {
                    expression: Expression::Infix {
                        left: Box::new(Expression::Ident("x".to_string())),
                        operator: Token::PLUS,
                        right: Box::new(Expression::Integer(2))
                    }
                }],
            }]
        );
        Ok(())
    }

    #[test]
    fn eval_function_application() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            let identity = fn(x) { x; }; identity(5);
            let identity = fn(x) { return x; }; identity(5);
            let double = fn(x) { x * 2; }; double(6);
            let add = fn(x, y) { x + y; }; add(6, 5);
            let add = fn(x, y) { x + y; }; add(6 + 5, add(6, 5));
            fn(x) { x; }(5);
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env).unwrap())
        .collect();
        assert_eq!(
            input,
            vec![
                Object::DeclareVariable,
                Object::Integer(5),
                Object::DeclareVariable,
                Object::Integer(5),
                Object::DeclareVariable,
                Object::Integer(12),
                Object::DeclareVariable,
                Object::Integer(11),
                Object::DeclareVariable,
                Object::Integer(22),
                Object::Integer(5),
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_array() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            [1, 2 * 2, 3 + 3];
            [1, 2 * 2, 3 + 3][1];
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env))
        .collect();
        assert_eq!(
            input,
            vec![
                Ok(Object::Array(vec![
                    Object::Integer(1),
                    Object::Integer(4),
                    Object::Integer(6),
                ])),
                Ok(Object::Integer(4))
            ]
        );
        Ok(())
    }

    #[test]
    fn eval_builtin() -> error::Result<()> {
        let mut env = Environment::new();
        let input: Vec<_> = Parser::new(Lexer::new(
            r#"
            len("");
            len("four");
            len("hello world");
            len(1);
            len("one", "two");
            "#,
        ))
        .parse_program()?
        .into_iter()
        .map(|x| eval(x, &mut env))
        .collect();
        assert_eq!(
            input,
            vec![
                Ok(Object::Integer(0)),
                Ok(Object::Integer(4)),
                Ok(Object::Integer(11)),
                Err(Error::EvalArgErr {
                    fnt_name: "len".to_string(),
                    got: Object::Integer(1)
                }),
                Err(Error::EvalParamNumErr {
                    expected: 1,
                    got: 2
                }),
            ]
        );
        Ok(())
    }
}
