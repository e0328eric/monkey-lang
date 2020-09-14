use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Object {
    Integer { value: i64 },
    Boolean { value: bool },
    ReturnValue { value: Box<Object> },
    Null,
}

impl Object {
    pub fn obj_type(&self) -> String {
        match self {
            Object::Integer { .. } => "INTEGER".to_string(),
            Object::Boolean { .. } => "BOOLEAN".to_string(),
            Object::ReturnValue { .. } => "RETURN".to_string(),
            Object::Null => "NULL".to_string(),
        }
    }

    pub fn is_same_type(left: &Object, right: &Object) -> bool {
        left.obj_type() == right.obj_type()
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer { value } => write!(f, "{}", value),
            Object::Boolean { value } => write!(f, "{}", value),
            Object::ReturnValue { value } => write!(f, "{}", *value),
            Object::Null => write!(f, "()"),
        }
    }
}
