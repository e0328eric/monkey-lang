#![allow(unused)]
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Opcode {
  OpConstant,
}

pub type Instructions = Vec<u8>;

pub struct Definition<'a> {
  name: &'a str,
  op_widths: &'a [usize],
}

impl Opcode {
  pub fn lookup(&self) -> Definition<'_> {
    match self {
      Self::OpConstant => Definition {
        name: "OpConstant",
        op_widths: &[2],
      },
    }
  }
}

impl Into<u8> for Opcode {
  fn into(self) -> u8 {
    match self {
      Self::OpConstant => 0,
    }
  }
}

#[allow(clippy::single_match)]
pub fn make(op: Opcode, operands: &[i64]) -> Vec<u8> {
  let def = op.lookup();
  let mut instruction_len = 1;

  for w in def.op_widths {
    instruction_len += w;
  }

  let mut instruction: Vec<u8> = Vec::with_capacity(instruction_len);
  instruction.push(op.into());

  let mut offset = 1;
  for (i, o) in operands.iter().enumerate() {
    let width = def.op_widths[i];
    match width {
      2 => {
        instruction.push(((*o as u16 & 0xff00) >> 8) as u8);
        instruction.push((*o as u16 & 0x00ff) as u8);
        offset += width;
      }
      _ => {}
    }
  }

  instruction
}

#[cfg(test)]
mod tests {
  use super::*;

  struct Expected<'a> {
    op: Opcode,
    operands: &'a [i64],
    expected: &'a [u8],
  }

  impl<'a> Expected<'a> {
    fn new(op: Opcode, operands: &'a [i64], expected: &'a [u8]) -> Self {
      Self {
        op,
        operands,
        expected,
      }
    }
  }

  #[test]
  fn test_make() {
    let tests = &[Expected::new(Opcode::OpConstant, &[65534], &[0, 255, 254])];

    for tt in tests {
      let instruction = make(tt.op, tt.operands);

      if instruction.len() != tt.expected.len() {
        panic!(
          "instruction has wrong length. want = {}, got = {}",
          tt.expected.len(),
          instruction.len()
        );
      }

      for (i, b) in tt.expected.iter().enumerate() {
        if instruction[i] != tt.expected[i] {
          panic!(
            "wrong byte at pos {}. want = {}, got = {}",
            i, b, instruction[i]
          );
        }
      }
    }
  }
}
