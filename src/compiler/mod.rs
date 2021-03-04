#![allow(unused)]
#[cfg(test)]
mod compiler_test;

use crate::code::{self, Instructions, Opcode};
use crate::error;
use crate::object::Object;
use crate::parser::ast::*;

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
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile(&mut self, prog: Program) -> error::Result<()> {
        for stmt in prog.get_stmts() {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }

    fn compile_stmt(&mut self, stmt: &Statement) -> error::Result<()> {
        Ok(())
    }

    fn compile_expression(&mut self, expr: Expression) -> error::Result<()> {
        Ok(())
    }
}
