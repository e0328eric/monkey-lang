#[derive(Debug, PartialEq, Eq)]
pub enum Object {
  Integer(i64),
  Boolean(bool),
  ReturnValue(Box<Object>),
  Error(String),
  DeclareVariable,
  Null,
}

impl Object {
  pub fn r#type(&self) -> &str {
    match self {
      Self::Integer(_) => "integer",
      Self::Boolean(_) => "boolean",
      Self::ReturnValue(_) => "return_value",
      Self::Error(_) => "error",
      Self::DeclareVariable => "declare",
      Self::Null => "null",
    }
  }
}
