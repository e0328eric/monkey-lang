use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Integer { value: i64 },
    Boolean { value: bool },
    ReturnValue { value: Box<Object> },
    DeclareVariable, // This special object makes not to print the value of expression
    Null,
}

impl Object {
    pub fn obj_type(&self) -> String {
        match self {
            Object::Integer { .. } => "INTEGER".to_string(),
            Object::Boolean { .. } => "BOOLEAN".to_string(),
            Object::ReturnValue { .. } => "RETURN".to_string(),
            Object::Null => "NULL".to_string(),
            Object::DeclareVariable => "DECLARE".to_string(),
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
            // If variables are declared, its variable should not displayed.
            // So I set this with unreachable.
            Object::DeclareVariable => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Environment {
    pub store: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: &str) -> Option<&Object> {
        self.store.get(name)
    }

    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val);
        Object::DeclareVariable
    }
}
