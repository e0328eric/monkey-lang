use crate::lexer::token::Token;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Object {
    Integer { value: i64 },
    Boolean { value: bool },
    ReturnValue { value: Box<Object> },
    Error { errorkind: ErrorKind },
    Null,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {}

impl Object {
    pub fn obj_type(&self) -> String {
        match self {
            Object::Integer { .. } => "BOOL".to_string(),
            Object::Boolean { .. } => "INTEGER".to_string(),
            Object::ReturnValue { .. } => "RETURN".to_string(),
            Object::Error { .. } => "ERROR".to_string(),
            Object::Null => "NULL".to_string(),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer { value } => write!(f, "{}", value),
            Object::Boolean { value } => write!(f, "{}", value),
            Object::ReturnValue { value } => write!(f, "{}", *value),
            Object::Error { .. } => write!(f, ""),
            Object::Null => write!(f, "()"),
        }
    }
}
