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
fn test_make() -> error::Result<()> {
    let tests = &[Expect::new(
        Opcode::OpConstant,
        &[65534],
        &[0x00, 0xff, 0xfe],
    )];

    for tt in tests {
        let instruction = tt.op.make(tt.operands)?;

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

    Ok(())
}

#[test]
fn test_instructions_string() -> error::Result<()> {
    let instructions = vec![
        Opcode::OpConstant.make(&[1]).unwrap(),
        Opcode::OpConstant.make(&[2]).unwrap(),
        Opcode::OpConstant.make(&[65535]).unwrap(),
    ];

    let expected = "0000 OpConstant 1\n0003 OpConstant 2\n0006 OpConstant 65535\n";
    let concatted = instructions.concat();

    assert_eq!(to_readable(concatted)?, expected);
    Ok(())
}

#[test]
fn test_read_operands() {
    struct ExpectedOperand<'a> {
        op: Opcode,
        operands: &'a [isize],
        bytes_read: isize,
    }

    let tests = &[ExpectedOperand {
        op: Opcode::OpConstant,
        operands: &[65535],
        bytes_read: 2,
    }];

    for tt in tests {
        let instruction = tt
            .op
            .make(tt.operands)
            .expect("Cannot unwrap this data that can be unwrapped.");
        let mut def = tt.op.lookup().unwrap();

        let (operand_read, n) = def.read_operands(&instruction[1..]);
        assert_eq!(n as isize, tt.bytes_read);

        for (i, want) in tt.operands.iter().enumerate() {
            assert_eq!(operand_read[i], *want);
        }
    }
}
