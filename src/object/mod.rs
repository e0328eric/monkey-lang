use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Object {
    Integer { value: i64 },
    Boolean { value: bool },
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer { value } => format!("{}", value),
            Object::Boolean { value } => format!("{}", value),
            Object::Null => "null".to_string(),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer { value } => write!(f, "{}", value),
            Object::Boolean { value } => write!(f, "{}", value),
            Object::Null => write!(f, "()"),
        }
    }
}
