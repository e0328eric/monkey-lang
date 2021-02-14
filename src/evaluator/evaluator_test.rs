use super::*;
use crate::error;
use crate::lexer::Lexer;
use crate::object::Object;
use crate::parser::Parser;

struct Expected<'a, T> {
  input: &'a str,
  expected: T,
}

impl<'a, T> Expected<'a, T> {
  fn new(input: &'a str, expected: T) -> Self {
    Self { input, expected }
  }
}

fn test_eval(input: &str) -> error::Result<Object> {
  let l = Lexer::new(input);
  let mut p = Parser::new(l);
  let program = p.parse_program();

  program.expect("Parse Error Occured!").eval()
}

macro_rules! make_test_case {
  ($test_name: ident | $($test_expected: expr),+ => $test_case: ident 2 arg) => {
    #[test]
    fn $test_name() -> error::Result<()> {
      let tests = &[
        $($test_expected,)+
      ];

      for tt in tests {
        let evaluated = test_eval(tt.input)?;
        $test_case(evaluated, tt.expected);
      }

      Ok(())
    }
  };
  ($test_name: ident | $($test_expected: expr),+ => $test_case: ident 1 arg with?) => {
    #[test]
    fn $test_name() -> error::Result<()> {
      let tests = &[
        $($test_expected,)+
      ];

      for tt in tests {
        $test_case(tt)?;
      }

      Ok(())
    }
  };
}

make_test_case!(test_eval_integer_expr |
  Expected::new("5", 5),
  Expected::new("10", 10),
  Expected::new("-5", -5),
  Expected::new("-10", -10),
  Expected::new("5 + 5 + 5 + 5 - 10", 10),
  Expected::new("2 * 2 * 2 * 2 * 2", 32),
  Expected::new("-50 + 100 + -50", 0),
  Expected::new("5 * 2 + 10", 20),
  Expected::new("5 + 2 * 10", 25),
  Expected::new("20 + 2 * -10", 0),
  Expected::new("50 / 2 * 2 + 10", 60),
  Expected::new("2 * (5 + 10)", 30),
  Expected::new("3 * 3 * 3 + 10", 37),
  Expected::new("3 * (3 * 3) + 10", 37),
  Expected::new("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50) =>
  test_integer_object 2 arg
);

make_test_case!(test_eval_bool_expr |
  Expected::new("true", true),
  Expected::new("false", false),
  Expected::new("1 < 2", true),
  Expected::new("1 > 2", false),
  Expected::new("1 < 1", false),
  Expected::new("1 > 1", false),
  Expected::new("1 == 1", true),
  Expected::new("1 != 1", false),
  Expected::new("1 == 2", false),
  Expected::new("1 != 2", true) =>
  test_bool_object 2 arg
);

make_test_case!(test_bang_operator |
  Expected::new("!true", false),
  Expected::new("!false", true),
  Expected::new("!5", false),
  Expected::new("!!true", true),
  Expected::new("!!false", false),
  Expected::new("!!5", true) =>
  test_bool_object 2 arg
);

make_test_case!(test_if_else_expresstions |
  Expected::new("if (true) { 10 }", Object::Integer(10)),
  Expected::new("if (false) { 10 }", NULL),
  Expected::new("if (1) { 10 }", Object::Integer(10)),
  Expected::new("if (1 < 2) { 10 }", Object::Integer(10)),
  Expected::new("if (1 > 2) { 10 }", NULL),
  Expected::new("if (1 < 2) { 10 } else { 20 }", Object::Integer(10)),
  Expected::new("if (1 > 2) { 10 } else { 20 }", Object::Integer(20)) =>
  test_if_case_helper 1 arg with?
);

make_test_case!(test_return_statements |
  Expected::new("return 10;", 10),
  Expected::new("return 10; 9;", 10),
  Expected::new("return 2 * 5; 9;", 10),
  Expected::new("9; return 2 * 5; 9;", 10),
  Expected::new("if (10 > 1) { if (10 > 1) { return 10; } return 1; }", 10) =>
  test_integer_object 2 arg
);

fn test_integer_object(evaluated: Object, expected: i64) {
  if let Object::Integer(n) = evaluated {
    assert_eq!(n, expected);
  } else {
    panic!("Object is not Integer. got = {:?}", evaluated);
  }
}

fn test_bool_object(evaluated: Object, expected: bool) {
  if let Object::Boolean(b) = evaluated {
    assert_eq!(b, expected);
  } else {
    panic!("Object is not Boolean. got = {:?}", evaluated);
  }
}

fn test_if_case_helper(tt: &Expected<'_, Object>) -> error::Result<()> {
  let evaluated = test_eval(tt.input)?;
  if let Object::Integer(n) = tt.expected {
    test_integer_object(evaluated, n);
  } else {
    assert_eq!(evaluated, NULL);
  }

  Ok(())
}
