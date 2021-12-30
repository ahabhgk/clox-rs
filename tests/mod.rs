use std::fmt;

use clox_rs::{Chunk, Parser, Scanner, VM};
use expect_test::{expect, Expect};

fn check(actual: &impl fmt::Debug, expect: Expect) {
  expect.assert_debug_eq(actual);
}

macro_rules! assert_snapshot {
  ($source:literal, $bytecodes:literal, $stack_snapshot:literal) => {
    let scanner = Scanner::new($source);
    let mut chunk = Chunk::new();
    let mut parser = Parser::new(scanner, &mut chunk);
    parser.advance().unwrap();
    parser.expression().unwrap();
    parser.end();
    check(&chunk, expect![[$bytecodes]]);
    let codes = chunk.codes.into_iter();
    let constants = chunk.constants.into_iter();
    let mut vm = VM::new(codes, constants);
    let inspector = vm.inspect().unwrap();
    check(&inspector, expect![[$stack_snapshot]])
  };
}

#[test]
fn chapter_17() {
  assert_snapshot!(
    "(-1 + 2) * 3 - -4",
    "
== Bytecodes ==
0000 Constant    0 '1'
0002 Negate
0003 Constant    1 '2'
0005 Add
0006 Constant    2 '3'
0008 Multiply
0009 Constant    3 '4'
0011 Negate
0012 Subtract
0013 Return

",
    "
== VM Stack Snapshot ==
[]
[1]
[-1]
[-1, 2]
[1]
[1, 3]
[3]
[3, 4]
[3, -4]
[7]

"
  );
}

#[test]
fn chapter_18() {
  assert_snapshot!(
    "!(5 - 4 > 3 * 2 == !nil)",
    "
== Bytecodes ==
0000 Constant    0 '5'
0002 Constant    1 '4'
0004 Subtract
0005 Constant    2 '3'
0007 Constant    3 '2'
0009 Multiply
0010 Greater
0011 Nil
0012 Not
0013 Equal
0014 Not
0015 Return

",
    "
== VM Stack Snapshot ==
[]
[5]
[5, 4]
[1]
[1, 3]
[1, 3, 2]
[1, 6]
[false]
[false, nil]
[false, true]
[false]
[true]

"
  );
}
