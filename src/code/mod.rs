#![allow(unused)]
#[cfg(test)]
mod code_test;

use std::collections::HashMap;

pub type Instructions = Vec<u8>;

pub fn to_readable(ins: Instructions) -> String {
    todo!();
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    OpConstant,
    Illigal = 0xFF,
}

pub struct Definition<'a> {
    name: &'static str,
    operand_widths: &'a [isize],
}

impl<'a> Definition<'a> {
    pub fn new(name: &'static str, operand_widths: &'a [isize]) -> Self {
        Self {
            name,
            operand_widths,
        }
    }

    #[allow(clippy::single_match)]
    pub fn read_operands(&mut self, ins: &'_ [u8]) -> (Vec<isize>, usize) {
        let mut operands = Vec::with_capacity(self.operand_widths.len());
        let mut offset = 0;

        for width in self.operand_widths {
            match width {
                2 => {
                    operands.push(read_usize_int(&ins[offset..]));
                    offset += *width as usize;
                }
                _ => {}
            }
        }

        (operands, offset)
    }
}

fn read_usize_int(ins: &[u8]) -> isize {
    todo!()
}

impl Opcode {
    pub fn lookup(&self) -> Option<Definition<'_>> {
        match self {
            Self::OpConstant => Some(Definition::new("OpConstant", &[2])),
            Self::Illigal => None,
        }
    }

    fn definitions(&self) -> Option<Definition<'_>> {
        self.lookup()
    }

    #[allow(clippy::single_match)]
    pub fn make(&self, operands: &[isize]) -> Option<Vec<u8>> {
        let def = self.definitions()?;
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

        Some(instruction)
    }
}

impl From<u8> for Opcode {
    fn from(source: u8) -> Self {
        Self::OpConstant
    }
}
