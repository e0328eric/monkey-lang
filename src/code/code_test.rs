use super::*;

struct Expect<'a> {
  op: Opcode,
  operands: &'a [isize],
  expected: &'a [u8],
}

impl<'a> Expect<'a> {
  fn new(op: Opcode, operands: &'a [isize], expected: &'a [u8]) -> Self {
    Self {
      op,
      operands,
      expected,
    }
  }
}

#[test]
fn test_make() {
  let tests = &[Expect::new(
    Opcode::OpConstant,
    &[65534],
    &[0x00, 0xff, 0xfe],
  )];

  for tt in tests {
    let instruction = tt.op.make(tt.operands);

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

#[test]
fn test_instructions_string() {
  let instructions = &[
    Opcode::OpConstant.make(&[1]),
    Opcode::OpConstant.make(&[2]),
    Opcode::OpConstant.make(&[65535]),
  ];

  let expected = "0000 OpConstant 1\n0003 OpConstant 2\n0006 OpConstant 65535";
}
