use std::fmt::{self, Debug, Display};

use crate::lexer::token::Token;
use crate::object::Object;

pub enum MonkeyErr {
    NoneErr, // This error is only for blank error
    // These two errors are critical errors so that the program panics
    IOErr(std::io::Error),
    FmtErr(fmt::Error),
    // These errors are pure monkey lang errors
    CannotConvertStringErr {
        got: Token,
    },
    CannotConvertSymbolErr {
        got: Token,
    },
    PrefixParseNoneErr {
        got: Token,
    },
    InfixParseNoneErr {
        got: Token,
    },
    ParseExprErr {
        expected: String,
        got: Token,
    },
    ParseTokDiffErr {
        expected: Token,
        got: Token,
    },
    EvalUnknownPrefix {
        operator: Token,
        right: Object,
    },
    EvalUnknownInfix {
        left: Object,
        operator: Token,
        right: Object,
    },
    EvalTypeMismatch {
        left: Object,
        operator: Token,
        right: Object,
    },
    EvalPowErr,
    EvalIdentNotFound {
        name_got: String,
    },
    EvalNotFunction,
    EvalArgErr {
        fnt_name: String,
        got: Object,
    },
    EvalParamNumErr {
        expected: usize,
        got: usize,
    },
}

impl MonkeyErr {
    pub fn is_critical_err(&self) -> bool {
        match self {
            MonkeyErr::IOErr(_) | MonkeyErr::FmtErr(_) => true,
            _ => false,
        }
    }
}

impl PartialEq for MonkeyErr {
    fn eq(&self, other: &Self) -> bool {
        use MonkeyErr::*;
        match (&self, other) {
            (NoneErr, NoneErr) => true,
            (IOErr(_), IOErr(_)) => true,
            (FmtErr(_), FmtErr(_)) => true,
            (CannotConvertStringErr { .. }, CannotConvertStringErr { .. }) => true,
            (CannotConvertSymbolErr { .. }, CannotConvertSymbolErr { .. }) => true,
            (PrefixParseNoneErr { .. }, PrefixParseNoneErr { .. }) => true,
            (InfixParseNoneErr { .. }, InfixParseNoneErr { .. }) => true,
            (ParseExprErr { .. }, ParseExprErr { .. }) => true,
            (ParseTokDiffErr { .. }, ParseTokDiffErr { .. }) => true,
            (EvalUnknownPrefix { .. }, EvalUnknownPrefix { .. }) => true,
            (EvalUnknownInfix { .. }, EvalUnknownInfix { .. }) => true,
            (EvalTypeMismatch { .. }, EvalTypeMismatch { .. }) => true,
            (EvalPowErr { .. }, EvalPowErr { .. }) => true,
            (EvalIdentNotFound { .. }, EvalIdentNotFound { .. }) => true,
            (EvalNotFunction, EvalNotFunction) => true,
            (EvalArgErr { .. }, EvalArgErr { .. }) => true,
            (EvalParamNumErr { .. }, EvalParamNumErr { .. }) => true,
            _ => false,
        }
    }
}

impl Display for MonkeyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonkeyErr::NoneErr => write!(f, "NoneError"),
            MonkeyErr::IOErr(ref e) => Display::fmt(e, f),
            MonkeyErr::FmtErr(ref e) => Display::fmt(e, f),
            MonkeyErr::CannotConvertStringErr { got } => {
                write!(f, "Cannot take string from {}", got.take_tok_name())
            }
            MonkeyErr::CannotConvertSymbolErr { got } => {
                write!(f, "Cannot convery symbol from {}", got.take_tok_name())
            }
            MonkeyErr::PrefixParseNoneErr { got } => write!(
                f,
                "Cannot take prefix function for {} found",
                got.take_tok_name()
            ),
            MonkeyErr::InfixParseNoneErr { got } => write!(
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
            MonkeyErr::EvalUnknownPrefix { operator, right } => write!(
                f,
                "Unknown operator: {0}{1}",
                operator.to_symbol().unwrap(),
                right.obj_type()
            ),
            MonkeyErr::EvalUnknownInfix {
                left,
                operator,
                right,
            } => write!(
                f,
                "Unknown operator: {0} {1} {2}",
                left.obj_type(),
                operator.to_symbol().unwrap(),
                right.obj_type(),
            ),
            MonkeyErr::EvalTypeMismatch {
                left,
                operator,
                right,
            } => write!(
                f,
                "Type mismatch: {0} {1} {2}",
                left.obj_type(),
                operator.to_symbol().unwrap(),
                right.obj_type(),
            ),
            MonkeyErr::EvalPowErr => write!(f, "Cannot power with negative value"),
            MonkeyErr::EvalIdentNotFound { name_got } => {
                write!(f, "Identifier not found: {}", name_got)
            }
            MonkeyErr::EvalNotFunction => write!(f, "Attempted to call non-function"),
            MonkeyErr::EvalArgErr { fnt_name, got } => write!(
                f,
                "Argument `{0}` not supported, got {1}",
                fnt_name,
                got.obj_type()
            ),
            MonkeyErr::EvalParamNumErr { expected, got } => write!(
                f,
                "Wrong number of arguments, got={1}, want={0}",
                expected, got
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

impl From<fmt::Error> for MonkeyErr {
    fn from(e: fmt::Error) -> Self {
        MonkeyErr::FmtErr(e)
    }
}

pub type Result<T> = std::result::Result<T, MonkeyErr>;

#[macro_export]
macro_rules! handle_error {
    ($handle: expr => $result: stmt) => {{
        if let Err(error) = $handle {
            if error.is_critical_err() {
                panic!("{}", error);
            } else {
                eprintln!("{}", error);
            }
        } else {
            $result
        }
    }};
}
