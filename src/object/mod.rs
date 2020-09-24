use crate::error::{self, MonkeyErr};
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
    ReturnValue(Box<Object>),
    Function {
        parameters: Vec<String>,
        body: BlockStmt,
    },
    BuiltinFnt(Builtin),
    DeclareVariable, // This special object makes not to print the value of an expression
    Null,
}

// To make unique objects
pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

impl Object {
    pub fn obj_type(&self) -> String {
        match self {
            Object::Integer { .. } => "INTEGER".to_string(),
            Object::String { .. } => "STRING".to_string(),
            Object::Complex { .. } => "COMPLEX".to_string(),
            Object::Boolean { .. } => "BOOLEAN".to_string(),
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
pub enum Builtin {
    ConvertErr, // This is only for error handle
    Len,
}

// Take an equality between Bultin and String
impl PartialEq<String> for Builtin {
    fn eq(&self, other: &String) -> bool {
        match other.as_str() {
            "len" => *self == Builtin::Len,
            _ => *self == Builtin::ConvertErr,
        }
    }
}

// This implementation needed because of proving the symmetric part
impl PartialEq<Builtin> for String {
    fn eq(&self, other: &Builtin) -> bool {
        other == self
    }
}

impl From<String> for Builtin {
    fn from(value: String) -> Self {
        match value.as_str() {
            "len" => Builtin::Len,
            _ => Builtin::ConvertErr,
        }
    }
}

impl Builtin {
    pub fn apply(&self, args: Vec<Object>) -> error::Result<Object> {
        match &self {
            Builtin::Len => {
                if args.len() != 1 {
                    return Err(MonkeyErr::EvalParamNumErr {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let arg = &args[0];
                if let Object::String(s) = arg {
                    Ok(Object::Integer(s.chars().count() as i64))
                } else {
                    Err(MonkeyErr::EvalArgErr {
                        fnt_name: "len".to_string(),
                        got: arg.clone(),
                    })
                }
            }
            Builtin::ConvertErr => Err(MonkeyErr::NoneErr),
            _ => Ok(NULL),
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
