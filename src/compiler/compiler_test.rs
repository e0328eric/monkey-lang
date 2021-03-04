use super::*;
use crate::error::MonkeyErr;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::any::Any;

macro_rules! emit_error {
  ($msg: expr) => {
    return Err(MonkeyErr::CompileErr {
      msg: format!($msg)
    });
  };
  ($msg: expr, $($args: expr),*) => {
    return Err(MonkeyErr::CompileErr {
      msg: format!($msg, $($args),*)
    });
  };
}

struct Expect<'a> {
    input: &'a str,
    expt_consts: &'a [&'a dyn Any],
    expt_instructions: Vec<Vec<u8>>,
}

impl<'a> Expect<'a> {
    fn new(
        input: &'a str,
        expt_consts: &'a [&'a dyn Any],
        expt_instructions: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            input,
            expt_consts,
            expt_instructions,
        }
    }
}

#[test]
fn test_integer_arithmetic() -> error::Result<()> {
    let tests: &[Expect<'_>] = &[Expect::new(
        "1 + 2",
        &[&1, &2],
        vec![
            Opcode::OpConstant.make(&[0]).unwrap(),
            Opcode::OpConstant.make(&[1]).unwrap(),
        ],
    )];

    run_compiler_test(tests)
}

fn run_compiler_test<'a>(tests: &'a [Expect<'a>]) -> error::Result<()> {
    for tt in tests {
        let program = Parser::new(Lexer::new(tt.input)).parse_program()?;

        let mut compiler = Compiler::new();
        compiler.compile(program)?;
        let bytecode = compiler.bytecode();

        test_instructions(&tt.expt_instructions, bytecode.instructions)?;

        test_constants(tt.expt_consts, &bytecode.constants)?;
    }

    Ok(())
}

fn parse(input: String) -> Program {
    let l = Lexer::new(&input);
    let mut p = Parser::new(l);

    // This unwrap() can be panic.
    // However, this function is used only by testing compiler.
    // So, if a panic occured, we can think that the test failed.
    p.parse_program().unwrap()
}

fn test_instructions(expect: &[Vec<u8>], actual: Instructions) -> error::Result<()> {
    let concatted = expect.concat();

    if actual.len() != concatted.len() {
        emit_error!(
            "wrong instructions length.\nwant = {:?}, got = {:?}",
            concatted,
            actual
        );
    }

    for (i, ins) in concatted.iter().enumerate() {
        if actual[i] != *ins {
            emit_error!(
                "wrong instructions at {}.\nwant = {:?}, got = {:?}",
                i,
                concatted,
                actual
            );
        }
    }

    Ok(())
}

fn test_constants(expect: &[&dyn Any], actual: &[Object]) -> error::Result<()> {
    if expect.len() != actual.len() {
        emit_error!(
            "wrong number of constants.\nwant = {}, got = {}",
            expect.len(),
            actual.len()
        );
    }

    for (i, constant) in expect.iter().enumerate() {
        if constant.is::<i16>() {
            test_integer_object(constant.downcast_ref::<i64>().unwrap(), &actual[i]);
        }
    }

    Ok(())
}

fn test_integer_object(expected: &i64, actual: &Object) -> error::Result<()> {
    if let Object::Integer(n) = actual {
        if n != expected {
            emit_error!("object has wrong value. got={}, want={}", n, expected);
        }
    } else {
        emit_error!("object is not Integer.");
    }

    Ok(())
}
