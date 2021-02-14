use crate::parser::ast::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone)]
pub enum Object {
  Integer(i64),
  Boolean(bool),
  ReturnValue(Box<Object>),
  Function(Box<FunctionObj>),
  DeclareVariable,
  Null,
}

impl PartialEq for Object {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Integer(n1), Self::Integer(n2)) => n1 == n2,
      (Self::Boolean(b1), Self::Boolean(b2)) => b1 == b2,
      (Self::ReturnValue(o1), Self::ReturnValue(o2)) => o1 == o2,
      (Self::DeclareVariable, Self::DeclareVariable) => true,
      (Self::Null, Self::Null) => true,
      _ => false,
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

impl Object {
  pub fn r#type(&self) -> &str {
    match self {
      Self::Integer(_) => "integer",
      Self::Boolean(_) => "boolean",
      Self::ReturnValue(_) => "return_value",
      Self::Function(_) => "function",
      Self::DeclareVariable => "declare",
      Self::Null => "null",
    }
  }
}

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
