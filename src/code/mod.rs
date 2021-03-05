#![allow(unused)]

#[cfg(test)]
mod code_test;

use crate::error::{self, MonkeyErr};
use std::collections::HashMap;

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
            let width = *width as usize;
            match width {
                2 => {
                    operands.push(read_u16(&ins[offset..offset + width]) as isize);
                    offset += width;
                }
                _ => {}
            }
        }

        (operands, offset)
    }

    #[allow(clippy::single_match)]
    fn fmt_operands(&mut self, operands: Vec<isize>) -> error::Result<String> {
        let operand_cnt = self.operand_widths.len();
        if operands.len() != operand_cnt {
            return Err(MonkeyErr::FmtOperandsInputLenErr {
                expected: operand_cnt,
                got: operands.len(),
            });
        }
        match operand_cnt {
            1 => Ok(format!("{} {}", self.name, operands[0])),
            _ => Err(MonkeyErr::UnhandledOperandCountErr {
                name: self.name.to_string(),
            }),
        }
    }
}

fn read_u16(ins: &[u8]) -> u16 {
    assert_eq!(ins.len(), 2);

    ((ins[0] as u16) << 8) | ins[1] as u16
}

impl Opcode {
    pub fn lookup(&self) -> error::Result<Definition<'_>> {
        match self {
            Self::OpConstant => Ok(Definition::new("OpConstant", &[2])),
            Self::Illigal => Err(MonkeyErr::IlligalOpcodeFoundErr),
        }
    }

    fn definitions(&self) -> error::Result<Definition<'_>> {
        self.lookup()
    }

    #[allow(clippy::single_match)]
    pub fn make(&self, operands: &[isize]) -> error::Result<Vec<u8>> {
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

        Ok(instruction)
    }
}

impl From<u8> for Opcode {
    fn from(source: u8) -> Self {
        Self::OpConstant
    }
}

pub type Instructions = Vec<u8>;

pub fn to_readable(ins: Instructions) -> error::Result<String> {
    let mut out = String::new();
    let mut i = 0;
    while i < ins.len() {
        let opcode = Opcode::from(ins[i]);
        let mut def = opcode.lookup()?;
        let (operands, read) = def.read_operands(&ins[i + 1..]);
        out += &format!("{:04} {}\n", i, def.fmt_operands(operands)?);

        i += 1 + read;
    }

    Ok(out)
}
