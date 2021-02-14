#[cfg(test)]
mod evaluator_test;

use crate::error;
use crate::lexer::token::Token;
use crate::object::builtin::BuiltInFnt;
use crate::object::*;
use crate::parser::ast::{BlockStmt, Expression, Program, Statement};
use std::cell::RefCell;
use std::rc::Rc;

type Error = error::MonkeyErr;

pub trait Evaluatable {
  fn eval(&self, env: &EnvWrapper) -> error::Result<Object>;
}

impl Evaluatable for Program {
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

impl Evaluatable for BlockStmt {
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

impl Evaluatable for Statement {
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

impl Evaluatable for Expression {
  fn eval(&self, env: &EnvWrapper) -> error::Result<Object> {
    match self {
      Self::Ident(string) => eval_ident(string, env),
      Self::Integer(n) => Ok(Object::Integer(*n)),
      Self::Boolean(b) => Ok(if *b { TRUE } else { FALSE }),
      Self::String(s) => Ok(Object::String(s.clone())),
      Self::Prefix { operator, right } => eval_prefix_expression(operator, (*right).eval(env)?),
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
  match (&left, &right) {
    (Object::Integer(_), Object::Integer(_)) => {
      eval_integer_infix_expression(left, operator, right)
    }
    (Object::String(_), Object::String(_)) => eval_string_infix_expression(left, operator, right),
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
