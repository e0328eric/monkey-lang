use crate::parser::ast::BlockStmt;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Integer {
        value: i64,
    },
    Complex {
        re: i64,
        im: i64,
    },
    Boolean {
        value: bool,
    },
    ReturnValue {
        value: Box<Object>,
    },
    Function {
        parameter: Vec<String>,
        body: BlockStmt,
        env: Box<Environment>,
    },
    DeclareVariable, // This special object makes not to print the value of an expression
    Null,
}

impl Object {
    pub fn obj_type(&self) -> String {
        match self {
            Object::Integer { .. } => "INTEGER".to_string(),
            Object::Complex { .. } => "COMPLEX".to_string(),
            Object::Boolean { .. } => "BOOLEAN".to_string(),
            Object::ReturnValue { .. } => "RETURN".to_string(),
            Object::Function { .. } => "FUNCTION".to_string(),
            Object::Null => "NULL".to_string(),
            Object::DeclareVariable => "DECLARE".to_string(),
        }
    }

    pub fn is_same_type(left: &Object, right: &Object) -> bool {
        left.obj_type() == right.obj_type()
    }

    pub fn to_complex(&self) -> Option<Self> {
        match self {
            Object::Complex { re, im } => Some(Object::Complex { re: *re, im: *im }),
            Object::Integer { value } => Some(Object::Complex { re: *value, im: 0 }),
            _ => None,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer { value } => write!(f, "{}", value),
            Object::Complex { re, im } => {
                if *re == 0 {
                    write!(f, "{}i", im)
                } else if *im >= 0 {
                    write!(f, "({0}+{1}i)", re, im)
                } else {
                    write!(f, "({0}{1}i)", re, im)
                }
            }
            Object::Boolean { value } => write!(f, "{}", value),
            Object::ReturnValue { value } => write!(f, "{}", *value),
            Object::Function { .. } => write!(f, ""),
            Object::Null => write!(f, "()"),
            // If variables are declared, its variable should not be displayed.
            // So I set this with unreachable.
            Object::DeclareVariable => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub store: HashMap<String, Object>,
    pub outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: &Environment) -> Self {
        let mut env = Environment::new();
        env.outer = Some(Box::new(outer.clone()));

        env
    }

    pub fn get(&mut self, name: &str) -> Option<&Object> {
        match self.store.get(name) {
            None => match self.outer {
                Some(ref mut env) => env.get(name),
                None => None,
            },
            Some(obj) => Some(obj),
        }
    }

    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val);
        Object::DeclareVariable
    }
}
