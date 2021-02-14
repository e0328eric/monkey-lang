use std::fmt::{self, Debug, Display};

use crate::lexer::token::Token;

pub enum MonkeyErr {
  // These two errors are critical errors so that the program panics
  IOErr(std::io::Error),
  FmtErr(fmt::Error),
  // These errors are pure monkey lang errors
  CannotConvertStringErr { got: Token },
  CannotConvertSymbolErr { got: Token },
  PrefixParseNoneErr { got: Token },
  InfixParseNoneErr { got: Token },
  ParseExprErr { expected: String, got: Token },
  ParseTokDiffErr { expected: Token, got: Token },
  EvalErr { msg: String },
}

impl MonkeyErr {
  pub fn is_critical_err(&self) -> bool {
    matches!(self, Self::IOErr(_) | Self::FmtErr(_))
  }
}

impl_partialeq!(
    MonkeyErr =>>
    IOErr(_),
    FmtErr(_),
    CannotConvertStringErr { .. },
    CannotConvertSymbolErr { .. },
    PrefixParseNoneErr { .. },
    InfixParseNoneErr { .. },
    ParseExprErr { .. },
    ParseTokDiffErr { .. },
    EvalErr { .. }
);

impl Display for MonkeyErr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IOErr(ref e) => Display::fmt(e, f),
      Self::FmtErr(ref e) => Display::fmt(e, f),
      Self::CannotConvertStringErr { got } => {
        write!(f, "Cannot take string from {}", got.take_tok_name())
      }
      Self::CannotConvertSymbolErr { got } => {
        write!(f, "Cannot convery symbol from {}", got.take_tok_name())
      }
      Self::PrefixParseNoneErr { got } => write!(
        f,
        "Cannot take prefix function for {} found",
        got.take_tok_name()
      ),
      Self::InfixParseNoneErr { got } => write!(
        f,
        "Cannot take infix function for {} found",
        got.take_tok_name()
      ),
      Self::ParseExprErr { expected, got } => write!(
        f,
        "Cannot parse {0} with {1}",
        expected,
        got.take_tok_name()
      ),
      Self::ParseTokDiffErr { expected, got } => write!(
        f,
        "Expected next token to be {0}, got {1} instead",
        expected.take_tok_name(),
        got.take_tok_name()
      ),
      Self::EvalErr { msg } => write!(f, "{}", msg),
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

impl From<fmt::Error> for MonkeyErr {
  fn from(e: fmt::Error) -> Self {
    MonkeyErr::FmtErr(e)
  }
}

pub type Result<T> = std::result::Result<T, MonkeyErr>;
