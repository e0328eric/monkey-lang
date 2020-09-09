use std::fmt::{self, Debug, Display};

use crate::token::Token;

pub enum MonkeyErr {
    IOErr(std::io::Error),
    PrefixNoneErr { got: Token },
    InfixNoneErr { got: Token },
    ParseExprErr { expected: String, got: Token },
    ParseTokDiffErr { expected: Token, got: Token },
}

impl Display for MonkeyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonkeyErr::IOErr(ref e) => Display::fmt(e, f),
            MonkeyErr::PrefixNoneErr { got } => write!(
                f,
                "Cannot take prefix function for {} found",
                got.take_tok_name()
            ),
            MonkeyErr::InfixNoneErr { got } => write!(
                f,
                "Cannot take infix function for {} found",
                got.take_tok_name()
            ),
            MonkeyErr::ParseExprErr { expected, got } => write!(
                f,
                "Cannot parse {0} with {1}",
                expected,
                got.take_tok_name()
            ),
            MonkeyErr::ParseTokDiffErr { expected, got } => write!(
                f,
                "Expected next token to be {0}, got {1} instead",
                expected.take_tok_name(),
                got.take_tok_name()
            ),
        }
    }
}

impl Debug for MonkeyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl From<std::io::Error> for MonkeyErr {
    fn from(e: std::io::Error) -> Self {
        MonkeyErr::IOErr(e)
    }
}

pub type Result<T> = std::result::Result<T, MonkeyErr>;
