pub mod builtin;

use crate::parser::ast::BlockStmt;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    String(String),
    Integer(i64),
    Complex(i64, i64),
    Boolean(bool),
    Array(Vec<Object>),
    ReturnValue(Box<Object>),
    Function {
        parameters: Vec<String>,
        body: BlockStmt,
    },
    BuiltinFnt(builtin::Builtin),
    DeclareVariable, // This special object makes not to print the value of an expression
    Null,
}

// To make unique objects
pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

// Transform boolean value to object bool
impl From<bool> for Object {
    fn from(boolean: bool) -> Self {
        if boolean {
            TRUE
        } else {
            FALSE
        }
    }
}

impl Object {
    pub fn obj_type(&self) -> String {
        match self {
            Object::Integer { .. } => "INTEGER".to_string(),
            Object::String { .. } => "STRING".to_string(),
            Object::Complex { .. } => "COMPLEX".to_string(),
            Object::Boolean { .. } => "BOOLEAN".to_string(),
            Object::Array { .. } => "ARRAY".to_string(),
            Object::ReturnValue { .. } => "RETURN".to_string(),
            Object::Function { .. } => "FUNCTION".to_string(),
            Object::BuiltinFnt { .. } => "BUILTIN".to_string(),
            Object::Null => "NULL".to_string(),
            Object::DeclareVariable => "DECLARE".to_string(),
        }
    }

    pub fn is_same_type(left: &Object, right: &Object) -> bool {
        left.obj_type() == right.obj_type()
    }

    pub fn to_complex(&self) -> Option<Self> {
        match self {
            Object::Complex(re, im) => Some(Object::Complex(*re, *im)),
            Object::Integer(value) => Some(Object::Complex(*value, 0)),
            _ => None,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String(value) => write!(f, "{}", value),
            Object::Integer(value) => write!(f, "{}", value),
            Object::Complex(re, im) => {
                if *re == 0 {
                    write!(f, "{}i", im)
                } else if *im >= 0 {
                    write!(f, "({0}+{1}i)", re, im)
                } else {
                    write!(f, "({0}{1}i)", re, im)
                }
            }
            Object::Boolean(value) => write!(f, "{}", value),
            Object::Array(lst) => {
                let mut print_str = String::from("[");
                for l in lst {
                    print_str += &format!("{}, ", l);
                }
                print_str.pop();
                print_str.pop();
                print_str += "]";
                write!(f, "{}", print_str)
            }
            Object::ReturnValue(value) => write!(f, "{}", *value),
            Object::Function { .. } => write!(f, ""),
            Object::BuiltinFnt { .. } => write!(f, ""),
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
    pub outer: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed_env(&self) -> Self {
        Self {
            store: HashMap::new(),
            outer: Some(Rc::new(self.clone())),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        if self.store.get(name).is_none() {
            if let Some(obj) = &self.outer {
                obj.get(name)
            } else {
                None
            }
        } else {
            self.store.get(name).cloned()
        }
    }

    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val);
        Object::DeclareVariable
    }
}
