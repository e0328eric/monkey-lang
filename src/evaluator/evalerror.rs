use crate::lexer::token::Token;
use crate::object::Object;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum EvalError {
    UnknownPrefix {
        operator: Token,
        right: Object,
    },
    UnknownInfix {
        left: Object,
        operator: Token,
        right: Object,
    },
    TypeMismatch {
        left: Object,
        operator: Token,
        right: Object,
    },
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UnknownPrefix { operator, right } => write!(
                f,
                "Unknown operator: {0}{1}",
                operator.to_symbol().unwrap(),
                right
            ),
            EvalError::UnknownInfix {
                left,
                operator,
                right,
            } => write!(
                f,
                "Unknown operator: {0} {1} {2}",
                left,
                operator.to_symbol().unwrap(),
                right
            ),
            EvalError::TypeMismatch {
                left,
                operator,
                right,
            } => write!(
                f,
                "Type mismatch: {0} {1} {2}",
                left,
                operator.to_symbol().unwrap(),
                right
            ),
        }
    }
}

pub type Result<T> = std::result::Result<T, EvalError>;
