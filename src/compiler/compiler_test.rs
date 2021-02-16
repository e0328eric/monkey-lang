use super::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

struct Expect<'a, T> {
  input: &'a str,
  expt_consts: &'a [T],
  expt_instructions: Vec<Instructions>,
}

impl<'a, T> Expect<'a, T> {
  fn new(input: &'a str, expt_consts: &'a [T], expt_instructions: Vec<Instructions>) -> Self {
    Self {
      input,
      expt_consts,
      expt_instructions,
    }
  }
}

#[test]
fn test_integer_arithmetic() -> error::Result<()> {
  let tests: &[Expect<'_, i64>] = &[Expect::new(
    "1 + 2",
    &[1, 2],
    vec![
      code::make(Opcode::OpConstant, &[0]),
      code::make(Opcode::OpConstant, &[1]),
    ],
  )];

  run_compiler_test(tests)
}

fn run_compiler_test<'a, T>(tests: &'a [Expect<'a, T>]) -> error::Result<()> {
  for tt in tests {
    let program = Parser::new(Lexer::new(tt.input)).parse_program()?;

    let mut compiler = Compiler::new();
    compiler.compile(program)?;
    let bytecode = Bytecode::from(compiler);

    // TODO: Implement this further
  }

  Ok(())
}
