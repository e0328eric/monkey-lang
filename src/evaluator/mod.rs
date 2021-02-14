#[cfg(test)]
mod evaluator_test;

use crate::error;
use crate::lexer::token::Token;
use crate::object::Object;
use crate::parser::ast::{BlockStmt, Expression, Program, Statement};

const TRUE: Object = Object::Boolean(true);
const FALSE: Object = Object::Boolean(false);
const NULL: Object = Object::Null;

pub trait Evaluatable {
  fn eval(&self) -> error::Result<Object>;
}

impl Evaluatable for Program {
  fn eval(&self) -> error::Result<Object> {
    let mut result = NULL;

    for stmt in self.get_stmts() {
      result = stmt.eval()?;
      if let Object::ReturnValue(value) = result {
        return Ok(*value);
      }
    }

    Ok(result)
  }
}

impl Evaluatable for BlockStmt {
  fn eval(&self) -> error::Result<Object> {
    let mut result = NULL;

    for stmt in self {
      result = stmt.eval()?;
      if let Object::ReturnValue(_) = result {
        return Ok(result);
      }
    }

    Ok(result)
  }
}

impl Evaluatable for Statement {
  fn eval(&self) -> error::Result<Object> {
    match self {
      Self::ExpressionStmt { expression } => expression.eval(),
      Self::ReturnStmt { value } => Ok(Object::ReturnValue(Box::new(value.eval()?))),
      _ => Ok(NULL),
    }
  }
}

impl Evaluatable for Expression {
  fn eval(&self) -> error::Result<Object> {
    match self {
      Self::Integer(n) => Ok(Object::Integer(*n)),
      Self::Boolean(b) => Ok(if *b { TRUE } else { FALSE }),
      Self::Prefix { operator, right } => Ok(eval_prefix_expression(operator, (*right).eval()?)),
      Self::IfExpr {
        condition,
        consequence,
        alternative,
      } => eval_if_expression((*condition).eval()?, consequence, alternative.as_ref()),
      Self::Infix {
        left,
        operator,
        right,
      } => Ok(eval_infix_expression(
        (*left).eval()?,
        operator,
        (*right).eval()?,
      )),
      _ => Ok(NULL),
    }
  }
}

fn eval_prefix_expression(operator: &Token, right: Object) -> Object {
  match operator {
    Token::BANG => {
      if right == FALSE {
        TRUE
      } else {
        FALSE
      }
    }
    Token::MINUS => {
      if let Object::Integer(n) = right {
        Object::Integer(-n)
      } else {
        NULL
      }
    }
    _ => Object::Error(format!(
      "unknown operator: {:?}{}",
      operator,
      right.r#type()
    )),
  }
}

fn eval_infix_expression(left: Object, operator: &Token, right: Object) -> Object {
  match (&left, &right) {
    (Object::Integer(_), Object::Integer(_)) => {
      eval_integer_infix_expression(left, operator, right)
    }
    _ if operator == &Token::EQ || operator == &Token::NOTEQ => {
      if left.r#type() != right.r#type() {
        Object::Error(format!(
          "type mismatch: {} {:?} {}",
          left.r#type(),
          operator,
          right.r#type()
        ))
      } else if left == right {
        TRUE
      } else {
        FALSE
      }
    }
    _ => Object::Error(format!(
      "unknown operator: {} {:?} {}",
      left.r#type(),
      operator,
      right.r#type(),
    )),
  }
}

fn eval_integer_infix_expression(left: Object, operator: &Token, right: Object) -> Object {
  let lv = if let Object::Integer(lv) = left {
    lv
  } else {
    unreachable!()
  };
  let rv = if let Object::Integer(rv) = right {
    rv
  } else {
    unreachable!()
  };
  match operator {
    Token::PLUS => Object::Integer(lv + rv),
    Token::MINUS => Object::Integer(lv - rv),
    Token::ASTERISK => Object::Integer(lv * rv),
    Token::SLASH => Object::Integer(lv / rv),
    Token::LT => Object::Boolean(lv < rv),
    Token::GT => Object::Boolean(lv > rv),
    Token::EQ => Object::Boolean(lv == rv),
    Token::NOTEQ => Object::Boolean(lv != rv),
    _ => Object::Error(format!(
      "unknown operator: {} {:?} {}",
      left.r#type(),
      operator,
      right.r#type()
    )),
  }
}

#[allow(clippy::ptr_arg)]
fn eval_if_expression(
  condition: Object,
  consequence: &BlockStmt,
  alternative: Option<&BlockStmt>,
) -> error::Result<Object> {
  if !matches!(condition, NULL | FALSE) {
    consequence.eval()
  } else if let Some(stmts) = alternative {
    stmts.eval()
  } else {
    Ok(NULL)
  }
}
