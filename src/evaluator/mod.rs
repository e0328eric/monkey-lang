#![recursion_limit = "500"]
#[cfg(test)]
mod evaluator_test;
pub mod gc;

use crate::error;
use crate::lexer::token::Token;
use crate::object::builtin::BuiltInFnt;
use crate::object::*;
use crate::parser::ast::{BlockStmt, Expression, Program, Statement};
use gc::{GCBox, GC};
use std::cell::RefCell;
use std::rc::Rc;

type Error = error::MonkeyErr;

pub struct Evaluator {
    gc_count: usize,
    gc: GC<Object>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            gc_count: 0,
            gc: GC {
                head: GCBox::new(NULL),
            },
        }
    }

    pub fn eval(&mut self, program: Program, env: &EnvWrapper) -> error::Result<GCBox<Object>> {
        let mut result = GCBox::new(NULL);

        for stmt in program.get_stmts() {
            result = self.eval_statement(stmt, env)?;
            if let Object::ReturnValue(value) = &*result {
                return Ok(value.clone());
            }
        }

        Ok(result)
    }

    fn eval_block_stmts(
        &mut self,
        stmts: &BlockStmt,
        env: &EnvWrapper,
    ) -> error::Result<GCBox<Object>> {
        let mut result = GCBox::new(NULL);
        for stmt in stmts {
            result = self.eval_statement(stmt, env)?;
            result.mark();
            if let Object::ReturnValue(_) = *result {
                return Ok(result);
            }
            self.gc_count += 1;
            if self.gc_count > 99 {
                GCBox::mark_env(&env);
                self.gc.sweep();
                self.gc_count = 0;
            }
        }

        Ok(result)
    }

    fn eval_statement(
        &mut self,
        stmt: &Statement,
        env: &EnvWrapper,
    ) -> error::Result<GCBox<Object>> {
        match stmt {
            Statement::ExpressionStmt { expression } => self.eval_expression(expression, env),
            Statement::ReturnStmt { value } => Ok(GCBox::new(Object::ReturnValue(
                self.eval_expression(value, env)?,
            ))),
            Statement::LetStmt { name, value } => {
                let val = self.eval_expression(value, env)?;
                env.borrow_mut().push(name.clone(), val);
                Ok(GCBox::new(Object::DeclareVariable))
            }
        }
    }

    fn eval_expression(
        &mut self,
        expr: &Expression,
        env: &EnvWrapper,
    ) -> error::Result<GCBox<Object>> {
        match expr {
            Expression::Ident(string) => self.eval_ident(string, env),
            Expression::Integer(n) => {
                let mut result = GCBox::new(Object::Integer(*n));
                self.gc.add(&mut result);
                Ok(result)
            }
            Expression::Boolean(b) => {
                let mut result = GCBox::new(if *b { TRUE } else { FALSE });
                self.gc.add(&mut result);
                Ok(result)
            }
            Expression::String(s) => {
                let mut result = GCBox::new(Object::String(s.clone()));
                self.gc.add(&mut result);
                Ok(result)
            }
            Expression::Array(array) => Ok(GCBox::new(Object::Array(
                self.eval_expressions(array, env)?,
            ))),
            Expression::Prefix { operator, right } => {
                let right = self.eval_expression(right, env)?;
                self.eval_prefix_expression(operator, right)
            }
            Expression::IfExpr {
                condition,
                consequence,
                alternative,
            } => {
                let condition = self.eval_expression(condition, env)?;
                self.eval_if_expression(condition, consequence, alternative.as_ref(), env)
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left = self.eval_expression(left, env)?;
                let right = self.eval_expression(right, env)?;
                self.eval_infix_expression(left, operator, right)
            }
            Expression::Function { parameters, body } => Ok(GCBox::new(Object::Function(
                FunctionObj::new(parameters, body, env),
            ))),
            Expression::Call {
                function,
                arguments,
            } => {
                let fnt = self.eval_expression(function, env)?;
                let args = self.eval_expressions(arguments, env)?;
                self.apply_function(fnt, args)
            }
            Expression::Hash { key, value } => self.eval_hash_literal(key, value, env),
            _ => Ok(GCBox::new(NULL)),
        }
    }

    fn eval_ident(&mut self, string: &str, env: &EnvWrapper) -> error::Result<GCBox<Object>> {
        let val = env.as_ref().borrow().get(string);
        if let Some(v) = val {
            return Ok(v);
        }

        let builtin: BuiltInFnt = string.into();
        if builtin != BuiltInFnt::NotBuiltIn {
            return Ok(GCBox::new(Object::BuiltIn(builtin)));
        }

        Err(Error::EvalErr {
            msg: format!("identifier not found: {}", string),
        })
    }

    fn eval_prefix_expression(
        &mut self,
        operator: &Token,
        right: GCBox<Object>,
    ) -> error::Result<GCBox<Object>> {
        match operator {
            Token::BANG => {
                if *right == FALSE {
                    let mut result = GCBox::new(TRUE);
                    self.gc.add(&mut result);
                    Ok(result)
                } else {
                    let mut result = GCBox::new(FALSE);
                    self.gc.add(&mut result);
                    Ok(result)
                }
            }
            Token::MINUS => {
                if let Object::Integer(n) = *right {
                    let mut result = GCBox::new(Object::Integer(-n));
                    self.gc.add(&mut result);
                    Ok(result)
                } else {
                    let mut result = GCBox::new(NULL);
                    self.gc.add(&mut result);
                    Ok(result)
                }
            }
            _ => Err(Error::EvalErr {
                msg: format!("unknown operator: {:?}{}", operator, right.r#type()),
            }),
        }
    }

    fn eval_infix_expression(
        &mut self,
        left: GCBox<Object>,
        operator: &Token,
        right: GCBox<Object>,
    ) -> error::Result<GCBox<Object>> {
        if operator == &Token::LBRACKET {
            return self.eval_index_expression(left, right);
        }
        match (&*left, &*right) {
            (Object::Integer(_), Object::Integer(_)) => {
                self.eval_integer_infix_expression(left, operator, right)
            }
            (Object::String(_), Object::String(_)) => {
                self.eval_string_infix_expression(left, operator, right)
            }
            _ if left.r#type() != right.r#type() => Err(Error::EvalErr {
                msg: format!(
                    "type mismatch: {} {:?} {}",
                    left.r#type(),
                    operator,
                    right.r#type()
                ),
            }),
            _ => match operator {
                Token::EQ => {
                    if *left == *right {
                        let mut result = GCBox::new(TRUE);
                        self.gc.add(&mut result);
                        Ok(result)
                    } else {
                        let mut result = GCBox::new(FALSE);
                        self.gc.add(&mut result);
                        Ok(result)
                    }
                }
                Token::NOTEQ => {
                    if *left != *right {
                        let mut result = GCBox::new(TRUE);
                        self.gc.add(&mut result);
                        Ok(result)
                    } else {
                        let mut result = GCBox::new(FALSE);
                        self.gc.add(&mut result);
                        Ok(result)
                    }
                }
                _ => Err(Error::EvalErr {
                    msg: format!(
                        "unknown operator: {} {:?} {}",
                        left.r#type(),
                        operator,
                        right.r#type(),
                    ),
                }),
            },
        }
    }

    fn eval_integer_infix_expression(
        &mut self,
        left: GCBox<Object>,
        operator: &Token,
        right: GCBox<Object>,
    ) -> error::Result<GCBox<Object>> {
        let lv = if let Object::Integer(lv) = &*left {
            *lv
        } else {
            unreachable!();
        };
        let rv = if let Object::Integer(rv) = &*right {
            *rv
        } else {
            unreachable!();
        };
        let mut result = GCBox::new(match operator {
            Token::PLUS => Object::Integer(lv + rv),
            Token::MINUS => Object::Integer(lv - rv),
            Token::ASTERISK => Object::Integer(lv * rv),
            Token::SLASH => Object::Integer(lv / rv),
            Token::LT => Object::Boolean(lv < rv),
            Token::GT => Object::Boolean(lv > rv),
            Token::EQ => Object::Boolean(lv == rv),
            Token::NOTEQ => Object::Boolean(lv != rv),
            _ => {
                return Err(Error::EvalErr {
                    msg: format!(
                        "unknown operator: {} {:?} {}",
                        left.r#type(),
                        operator,
                        right.r#type()
                    ),
                })
            }
        });
        self.gc.add(&mut result);
        Ok(result)
    }

    fn eval_string_infix_expression(
        &mut self,
        left: GCBox<Object>,
        operator: &Token,
        right: GCBox<Object>,
    ) -> error::Result<GCBox<Object>> {
        if operator != &Token::PLUS {
            return Err(Error::EvalErr {
                msg: format!(
                    "unknown operator: {} {:?} {}",
                    left.r#type(),
                    operator,
                    right.r#type()
                ),
            });
        }

        let left_val = if let Object::String(s) = &*left {
            s
        } else {
            unreachable!();
        };
        let right_val = if let Object::String(s) = &*right {
            s
        } else {
            unreachable!();
        };

        let mut result = GCBox::new(Object::String(left_val.clone() + right_val));
        self.gc.add(&mut result);
        Ok(result)
    }

    fn eval_index_expression(
        &mut self,
        left: GCBox<Object>,
        right: GCBox<Object>,
    ) -> error::Result<GCBox<Object>> {
        match (&*left, &*right) {
            (Object::Array(array), Object::Integer(idx)) => {
                let max = (array.len() - 1) as i64;

                if *idx < 0 || *idx > max {
                    return Ok(GCBox::new(NULL));
                }
                Ok(array[*idx as usize].clone())
            }
            (Object::Hash(hash), _) => {
                if let Some(hash_key) = right.hash_key() {
                    let pair = hash.pairs.get(&hash_key);
                    if let Some(hash_pair) = pair {
                        return Ok(hash_pair.value());
                    }
                    return Ok(GCBox::new(NULL));
                }
                return Err(Error::EvalErr {
                    msg: format!("unusable as hash key: {}", right.r#type()),
                });
            }
            _ => Err(Error::EvalErr {
                msg: format!("index operator not supported: {}", left.r#type()),
            }),
        }
    }

    #[allow(clippy::ptr_arg)]
    fn eval_if_expression(
        &mut self,
        condition: GCBox<Object>,
        consequence: &BlockStmt,
        alternative: Option<&BlockStmt>,
        env: &EnvWrapper,
    ) -> error::Result<GCBox<Object>> {
        if *condition != NULL && *condition != FALSE {
            self.eval_block_stmts(consequence, env)
        } else if let Some(stmts) = alternative {
            self.eval_block_stmts(stmts, env)
        } else {
            Ok(GCBox::new(NULL))
        }
    }

    fn eval_expressions(
        &mut self,
        arguments: &[Expression],
        env: &EnvWrapper,
    ) -> error::Result<Vec<GCBox<Object>>> {
        let mut result: Vec<GCBox<Object>> = Vec::new();

        for exp in arguments {
            result.push(self.eval_expression(exp, env)?);
        }

        Ok(result)
    }

    fn apply_function(
        &mut self,
        fnt: GCBox<Object>,
        args: Vec<GCBox<Object>>,
    ) -> error::Result<GCBox<Object>> {
        match &*fnt {
            Object::Function(f) => {
                let function = f;
                let extended_env = self.extended_fnt_env(&function, args)?;
                let evaluated = self.eval_block_stmts(function.get_body(), &extended_env)?;
                Ok(Self::unwrap_return_value(evaluated))
            }
            Object::BuiltIn(built) => Ok(built.call(args)?),
            _ => Err(Error::EvalErr {
                msg: format!("Not a function: {:?}", fnt.r#type()),
            }),
        }
    }

    fn extended_fnt_env(
        &mut self,
        function: &FunctionObj,
        mut args: Vec<GCBox<Object>>,
    ) -> error::Result<EnvWrapper> {
        let strong_env = if let Some(e) = function.get_env().upgrade() {
            e
        } else {
            panic!("Weak pointer is broken...");
        };
        let mut env = Environment::new_enclosed_env(&strong_env);

        if function.get_param().len() != args.len() {
            return Err(Error::EvalErr {
                msg: format!(
                    "Function does not take appropriate argument numbers. expected = {}, got = {}",
                    function.get_param().len(),
                    args.len(),
                ),
            });
        }

        for (_, param) in function.get_param().iter().enumerate() {
            env.push(param.clone(), args.remove(0));
        }

        Ok(Rc::new(RefCell::new(env)))
    }

    fn unwrap_return_value(evaluated: GCBox<Object>) -> GCBox<Object> {
        if let Object::ReturnValue(obj) = &*evaluated {
            obj.clone()
        } else {
            evaluated
        }
    }

    fn eval_hash_literal(
        &mut self,
        key: &[Expression],
        value: &[Expression],
        env: &EnvWrapper,
    ) -> error::Result<GCBox<Object>> {
        let pairs = key.iter().zip(value.iter());
        let mut hash: GCBox<Hash> = GCBox::new(Hash::default());

        for (k, v) in pairs {
            let key = self.eval_expression(k, env)?;

            if key.hash_key().is_none() {
                return Err(Error::EvalErr {
                    msg: format!("unusable as hash key: {}", key.r#type()),
                });
            }

            let value = self.eval_expression(v, env)?;
            let hashed = key.hash_key().unwrap();

            hash.pairs.insert(hashed, HashPair::new(key, value));
        }

        Ok(GCBox::new(Object::Hash(hash)))
    }
}

/*
pub trait Evaluable {
    fn eval(&self, env: &EnvWrapper) -> error::Result<Object>;
}

impl Evaluable for Program {
    fn eval(&self, env: &EnvWrapper) -> error::Result<Object> {
        let mut result = NULL;

        for stmt in self.get_stmts() {
            result = stmt.eval(env)?;
            if let Object::ReturnValue(value) = result {
                return Ok(*value);
            }
        }

        Ok(result)
    }
}

impl Evaluable for BlockStmt {
    fn eval(&self, env: &EnvWrapper) -> error::Result<Object> {
        let mut result = NULL;

        for stmt in self {
            result = stmt.eval(env)?;
            if let Object::ReturnValue(_) = result {
                return Ok(result);
            }
        }

        Ok(result)
    }
}

impl Evaluable for Statement {
    fn eval(&self, env: &EnvWrapper) -> error::Result<Object> {
        match self {
            Self::ExpressionStmt { expression } => expression.eval(env),
            Self::ReturnStmt { value } => Ok(Object::ReturnValue(Box::new(value.eval(env)?))),
            Self::LetStmt { name, value } => {
                let val = value.eval(env)?;
                env.borrow_mut().push(name.clone(), val);
                Ok(Object::DeclareVariable)
            }
        }
    }
}

impl Evaluable for Expression {
    fn eval(&self, env: &EnvWrapper) -> error::Result<Object> {
        match self {
            Self::Ident(string) => eval_ident(string, env),
            Self::Integer(n) => Ok(Object::Integer(*n)),
            Self::Boolean(b) => Ok(if *b { TRUE } else { FALSE }),
            Self::String(s) => Ok(Object::String(s.clone())),
            Self::Array(array) => Ok(Object::Array(eval_expressions(array, env)?)),
            Self::Prefix { operator, right } => {
                eval_prefix_expression(operator, (*right).eval(env)?)
            }
            Self::IfExpr {
                condition,
                consequence,
                alternative,
            } => eval_if_expression(
                (*condition).eval(env)?,
                consequence,
                alternative.as_ref(),
                env,
            ),
            Self::Infix {
                left,
                operator,
                right,
            } => eval_infix_expression((*left).eval(env)?, operator, (*right).eval(env)?),
            Self::Function { parameters, body } => {
                Ok(Object::Function(FunctionObj::new(parameters, body, env)))
            }
            Self::Call {
                function,
                arguments,
            } => {
                let fnt = function.eval(env)?;
                let args = eval_expressions(arguments, env)?;
                apply_function(fnt, args)
            }
            Self::Hash { key, value } => eval_hash_literal(key, value, env),
            _ => Ok(NULL),
        }
    }
}

fn eval_ident(string: &str, env: &EnvWrapper) -> error::Result<Object> {
    let val = env.as_ref().borrow().get(string);
    if let Some(v) = val {
        return Ok(v);
    }

    let builtin: BuiltInFnt = string.into();
    if builtin != BuiltInFnt::NotBuiltIn {
        return Ok(Object::BuiltIn(builtin));
    }

    Err(Error::EvalErr {
        msg: format!("identifier not found: {}", string),
    })
}

fn eval_prefix_expression(operator: &Token, right: Object) -> error::Result<Object> {
    match operator {
        Token::BANG => {
            if right == FALSE {
                Ok(TRUE)
            } else {
                Ok(FALSE)
            }
        }
        Token::MINUS => {
            if let Object::Integer(n) = right {
                Ok(Object::Integer(-n))
            } else {
                Ok(NULL)
            }
        }
        _ => Err(Error::EvalErr {
            msg: format!("unknown operator: {:?}{}", operator, right.r#type()),
        }),
    }
}

fn eval_infix_expression(left: Object, operator: &Token, right: Object) -> error::Result<Object> {
    if operator == &Token::LBRACKET {
        return eval_index_expression(left, right);
    }
    match (&left, &right) {
        (Object::Integer(_), Object::Integer(_)) => {
            eval_integer_infix_expression(left, operator, right)
        }
        (Object::String(_), Object::String(_)) => {
            eval_string_infix_expression(left, operator, right)
        }
        _ if left.r#type() != right.r#type() => Err(Error::EvalErr {
            msg: format!(
                "type mismatch: {} {:?} {}",
                left.r#type(),
                operator,
                right.r#type()
            ),
        }),
        _ => match operator {
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
            _ => Err(Error::EvalErr {
                msg: format!(
                    "unknown operator: {} {:?} {}",
                    left.r#type(),
                    operator,
                    right.r#type(),
                ),
            }),
        },
    }
}

fn eval_integer_infix_expression(
    left: Object,
    operator: &Token,
    right: Object,
) -> error::Result<Object> {
    let lv = if let Object::Integer(lv) = left {
        lv
    } else {
        unreachable!();
    };
    let rv = if let Object::Integer(rv) = right {
        rv
    } else {
        unreachable!();
    };
    match operator {
        Token::PLUS => Ok(Object::Integer(lv + rv)),
        Token::MINUS => Ok(Object::Integer(lv - rv)),
        Token::ASTERISK => Ok(Object::Integer(lv * rv)),
        Token::SLASH => Ok(Object::Integer(lv / rv)),
        Token::LT => Ok(Object::Boolean(lv < rv)),
        Token::GT => Ok(Object::Boolean(lv > rv)),
        Token::EQ => Ok(Object::Boolean(lv == rv)),
        Token::NOTEQ => Ok(Object::Boolean(lv != rv)),
        _ => Err(Error::EvalErr {
            msg: format!(
                "unknown operator: {} {:?} {}",
                left.r#type(),
                operator,
                right.r#type()
            ),
        }),
    }
}

fn eval_string_infix_expression(
    left: Object,
    operator: &Token,
    right: Object,
) -> error::Result<Object> {
    if operator != &Token::PLUS {
        return Err(Error::EvalErr {
            msg: format!(
                "unknown operator: {} {:?} {}",
                left.r#type(),
                operator,
                right.r#type()
            ),
        });
    }

    let left_val = if let Object::String(s) = left {
        s
    } else {
        unreachable!();
    };
    let right_val = if let Object::String(s) = right {
        s
    } else {
        unreachable!();
    };

    Ok(Object::String(left_val + &right_val))
}

fn eval_index_expression(left: Object, right: Object) -> error::Result<Object> {
    match (&left, &right) {
        (Object::Array(array), Object::Integer(idx)) => {
            let max = (array.len() - 1) as i64;

            if *idx < 0 || *idx > max {
                return Ok(NULL);
            }
            Ok(array[*idx as usize].clone())
        }
        (Object::Hash(hash), _) => {
            if let Some(hash_key) = right.hash_key() {
                let pair = hash.pairs.get(&hash_key);
                if let Some(hash_pair) = pair {
                    return Ok(hash_pair.value());
                }
                return Ok(NULL);
            }
            return Err(Error::EvalErr {
                msg: format!("unusable as hash key: {}", right.r#type()),
            });
        }
        _ => Err(Error::EvalErr {
            msg: format!("index operator not supported: {}", left.r#type()),
        }),
    }
}

#[allow(clippy::ptr_arg)]
fn eval_if_expression(
    condition: Object,
    consequence: &BlockStmt,
    alternative: Option<&BlockStmt>,
    env: &EnvWrapper,
) -> error::Result<Object> {
    if condition != NULL && condition != FALSE {
        consequence.eval(env)
    } else if let Some(stmts) = alternative {
        stmts.eval(env)
    } else {
        Ok(NULL)
    }
}
fn eval_expressions(arguments: &[Expression], env: &EnvWrapper) -> error::Result<Vec<Object>> {
    let mut result: Vec<Object> = Vec::new();

    for exp in arguments {
        result.push(exp.eval(env)?);
    }

    Ok(result)
}

fn apply_function(fnt: Object, args: Vec<Object>) -> error::Result<Object> {
    match fnt {
        Object::Function(f) => {
            let function = f;
            let extended_env = extended_fnt_env(&function, args)?;
            let evaluated = function.get_body().eval(&extended_env)?;
            Ok(unwrap_return_value(evaluated))
        }
        Object::BuiltIn(built) => Ok(built.call(args)?),
        _ => Err(Error::EvalErr {
            msg: format!("Not a function: {:?}", fnt.r#type()),
        }),
    }
}

fn extended_fnt_env(function: &FunctionObj, mut args: Vec<Object>) -> error::Result<EnvWrapper> {
    let strong_env = if let Some(e) = function.get_env().upgrade() {
        e
    } else {
        panic!("Weak pointer is broken...");
    };
    let mut env = Environment::new_enclosed_env(&strong_env);

    if function.get_param().len() != args.len() {
        return Err(Error::EvalErr {
            msg: format!(
                "Function does not take appropriate argument numbers. expected = {}, got = {}",
                function.get_param().len(),
                args.len(),
            ),
        });
    }

    for (_, param) in function.get_param().iter().enumerate() {
        env.push(param.clone(), args.remove(0));
    }

    Ok(Rc::new(RefCell::new(env)))
}

fn unwrap_return_value(evaluated: Object) -> Object {
    if let Object::ReturnValue(obj) = evaluated {
        *obj
    } else {
        evaluated
    }
}

fn eval_hash_literal(
    key: &[Expression],
    value: &[Expression],
    env: &EnvWrapper,
) -> error::Result<Object> {
    let pairs = key.iter().zip(value.iter());
    let mut hash: Box<Hash> = Box::new(Hash::default());

    for (k, v) in pairs {
        let key = k.eval(env)?;

        if key.hash_key().is_none() {
            return Err(Error::EvalErr {
                msg: format!("unusable as hash key: {}", key.r#type()),
            });
        }

        let value = v.eval(env)?;
        let hashed = key.hash_key().unwrap();

        hash.pairs.insert(hashed, HashPair::new(key, value));
    }

    Ok(Object::Hash(hash))
}
*/
