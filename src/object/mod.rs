pub mod builtin;

use crate::parser::ast::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Default, Debug, Clone)]
pub struct Environment {
  store: Vec<(String, Object)>,
  outer: Option<EnvWrapper>,
}

pub type EnvWrapper = Rc<RefCell<Environment>>;
pub type EnvWeakWrapper = Weak<RefCell<Environment>>;

impl Environment {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn new_enclosed_env(outer: &EnvWrapper) -> Self {
    let mut env = Environment::new();
    env.outer = Some(Rc::clone(outer));
    env
  }

  pub fn push(&mut self, ident: String, obj: Object) {
    self.store.push((ident, obj));
  }

  pub fn get(&self, ident: &str) -> Option<Object> {
    let obj = self
      .store
      .iter()
      .find(|(i, _)| i == ident)
      .map(|(_, o)| o)
      .cloned();
    if obj.is_none() {
      return self.outer.as_ref()?.as_ref().borrow().get(ident);
    }
    obj
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Hash {
  pub(crate) pairs: HashMap<HashKey, HashPair>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HashKey {
  obj_type: String,
  value: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HashPair {
  key: Object,
  value: Object,
}

impl HashPair {
  pub fn new(key: Object, value: Object) -> Self {
    Self { key, value }
  }

  pub fn value(&self) -> Object {
    self.value.clone()
  }
}

#[derive(Debug, Clone)]
pub enum Object {
  Integer(i64),
  Boolean(bool),
  String(String),
  ReturnValue(Box<Object>),
  Function(Box<FunctionObj>),
  Array(Vec<Object>),
  Hash(Box<Hash>),
  BuiltIn(builtin::BuiltInFnt),
  DeclareVariable,
  Null,
}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

impl PartialEq for Object {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Integer(n1), Self::Integer(n2)) => n1 == n2,
      (Self::Boolean(b1), Self::Boolean(b2)) => b1 == b2,
      (Self::String(s1), Self::String(s2)) => s1 == s2,
      (Self::ReturnValue(o1), Self::ReturnValue(o2)) => o1 == o2,
      (Self::Array(a1), Self::Array(a2)) => a1 == a2,
      (Self::Hash(h1), Self::Hash(h2)) => h1 == h2,
      (Self::BuiltIn(b1), Self::BuiltIn(b2)) => b1 == b2,
      (Self::DeclareVariable, Self::DeclareVariable) => true,
      (Self::Null, Self::Null) => true,
      _ => false,
    }
  }
}

impl Object {
  pub fn r#type(&self) -> &str {
    match self {
      Self::Integer(_) => "integer",
      Self::Boolean(_) => "boolean",
      Self::String(_) => "string",
      Self::ReturnValue(_) => "return_value",
      Self::Function(_) => "function",
      Self::Array(_) => "array",
      Self::Hash(_) => "hash",
      Self::BuiltIn(built) => (*built).into(),
      Self::DeclareVariable => "declare",
      Self::Null => "null",
    }
  }

  pub fn hash_key(&self) -> Option<HashKey> {
    match self {
      Self::Boolean(b) => Some(HashKey {
        obj_type: String::from("boolean"),
        value: if *b { 1 } else { 0 },
      }),
      Self::Integer(n) => Some(HashKey {
        obj_type: String::from("integer"),
        value: *n as u64,
      }),
      Self::String(string) => {
        let bytes = string.bytes();
        let mut sum: u64 = 0;
        for b in bytes {
          sum += sum.overflowing_add(b.into()).0;
        }

        Some(HashKey {
          obj_type: String::from("string"),
          value: sum,
        })
      }
      _ => None,
    }
  }
}
#[derive(Debug, Clone)]
pub struct FunctionObj {
  parameters: Vec<String>,
  body: BlockStmt,
  env: EnvWeakWrapper,
}

impl FunctionObj {
  #[allow(clippy::ptr_arg)]
  pub fn new(parameters: &Vec<String>, body: &BlockStmt, env: &EnvWrapper) -> Box<Self> {
    Box::new(Self {
      parameters: parameters.clone(),
      body: body.clone(),
      env: Rc::downgrade(env),
    })
  }

  pub fn get_param(&self) -> &Vec<String> {
    &self.parameters
  }

  pub fn get_body(&self) -> &BlockStmt {
    &self.body
  }

  pub fn get_env(&self) -> &EnvWeakWrapper {
    &self.env
  }
}
