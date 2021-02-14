use super::{Object, NULL};
use crate::error::{self, MonkeyErr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltInFnt {
  NotBuiltIn,
  Len,
}

impl From<&str> for BuiltInFnt {
  fn from(input: &str) -> Self {
    match input {
      "len" => Self::Len,
      _ => Self::NotBuiltIn,
    }
  }
}

impl Into<&str> for BuiltInFnt {
  fn into(self) -> &'static str {
    match self {
      Self::Len => "len",
      _ => "",
    }
  }
}

macro_rules! check_arg_len {
  ($args: ident, $num: expr) => {
    if $args.len() != $num {
      return Err(MonkeyErr::EvalErr {
        msg: format!(
          "Wrong number of arguments. got = {}, want = {}",
          $args.len(),
          $num
        ),
      });
    }
  };
}

impl BuiltInFnt {
  pub fn call(&self, args: Vec<Object>) -> error::Result<Object> {
    match self {
      Self::Len => {
        check_arg_len!(args, 1);

        let arg = &args[0];
        match arg {
          Object::String(s) => Ok(Object::Integer(s.len() as i64)),
          _ => Err(MonkeyErr::EvalErr {
            msg: format!("Argument to `len` not supported, got {}", arg.r#type()),
          }),
        }
      }
      _ => Ok(NULL),
    }
  }
}
