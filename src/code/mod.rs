#![allow(unused)]
#[cfg(test)]
mod code_test;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
  OpConstant,
}

pub type Instructions = Vec<Opcode>;

impl Opcode {
  pub fn lookup(&self) -> Definition<'_> {
    match self {
      Self::OpConstant => Definition::new("OpConstant", &[2]),
    }
  }

  fn definitions(&self) -> Definition<'_> {
    self.lookup()
  }

  #[allow(clippy::single_match)]
  pub fn make(&self, operands: &[isize]) -> Vec<u8> {
    let def = self.definitions();
    let mut instruction_len = 1usize;

    for w in def.operand_widths {
      instruction_len += *w as usize;
    }

    let mut instruction = Vec::with_capacity(instruction_len);
    instruction.push(*self as u8);
    let mut offset = 1usize;

    for (i, o) in operands.iter().enumerate() {
      let width = def.operand_widths[i];
      match width {
        2 => {
          instruction.push((((*o as u16) & 0xff00) >> 8) as u8);
          instruction.push(((*o as u16) & 0x00ff) as u8);
        }
        _ => {}
      }
      offset += width as usize;
    }

    instruction
  }
}

impl From<u8> for Opcode {
  fn from(source: u8) -> Self {
    Self::OpConstant
  }
}

pub struct Definition<'a> {
  name: &'static str,
  operand_widths: &'a [isize],
}

impl<'a> Definition<'a> {
  fn new(name: &'static str, operand_widths: &'a [isize]) -> Self {
    Self {
      name,
      operand_widths,
    }
  }
}
