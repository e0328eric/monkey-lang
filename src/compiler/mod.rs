#[cfg(test)]
mod compiler_test;

use crate::code::{self, Definition, Instructions, Opcode};
use crate::error::{self, MonkeyErr};
use crate::object::Object;
use crate::parser::ast::Program;

#[derive(Default)]
pub struct Compiler {
  instructions: Instructions,
  constants: Vec<Object>,
}

pub struct Bytecode {
  instructions: Instructions,
  constants: Vec<Object>,
}

impl Compiler {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn compile(&mut self, node: Program) -> error::Result<()> {
    Err(MonkeyErr::CompileErr { msg: String::new() })
  }
}

impl From<Compiler> for Bytecode {
  fn from(comp: Compiler) -> Self {
    Self {
      instructions: comp.instructions,
      constants: comp.constants,
    }
  }
}
