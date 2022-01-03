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
    parser.program().unwrap();
    parser.end();
    check(&chunk, expect![[$bytecodes]]);
    let mut vm = VM::new(chunk);
    let inspector = vm.inspect().unwrap();
    check(&inspector, expect![[$stack_snapshot]])
  };
}

#[test]
fn chapter_17() {
  assert_snapshot!(
    "(-1 + 2) * 3 - -4;",
    "
== Bytecodes ==
0000 Constant            0 '1'
0002 Negate
0003 Constant            1 '2'
0005 Add
0006 Constant            2 '3'
0008 Multiply
0009 Constant            3 '4'
0011 Negate
0012 Subtract
0013 Pop
0014 Return

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
[]

"
  );
}

#[test]
fn chapter_18() {
  assert_snapshot!(
    "!(5 - 4 > 3 * 2 == !nil);",
    "
== Bytecodes ==
0000 Constant            0 '5'
0002 Constant            1 '4'
0004 Subtract
0005 Constant            2 '3'
0007 Constant            3 '2'
0009 Multiply
0010 Greater
0011 Nil
0012 Not
0013 Equal
0014 Not
0015 Pop
0016 Return

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
[]

"
  );
}

#[test]
fn chapter_19() {
  assert_snapshot!(
    r#""aha" + "b";"#,
    r#"
== Bytecodes ==
0000 Constant            0 '"aha"'
0002 Constant            1 '"b"'
0004 Add
0005 Pop
0006 Return

"#,
    r#"
== VM Stack Snapshot ==
[]
["aha"]
["aha", "b"]
["ahab"]
[]

"#
  );
}

#[test]
fn chapter_21_print_statement() {
  assert_snapshot!(
    r#"
print 1 + 2;
print 3 * 4;
"#,
    r#"
== Bytecodes ==
0000 Constant            0 '1'
0002 Constant            1 '2'
0004 Add
0005 Print
0006 Constant            2 '3'
0008 Constant            3 '4'
0010 Multiply
0011 Print
0012 Return

"#,
    r#"
== VM Stack Snapshot ==
[]
[1]
[1, 2]
[3]
[]
[3]
[3, 4]
[12]
[]

"#
  );
}

#[test]
fn chapter_21_var_uninit() {
  assert_snapshot!(
    r#"var a;"#,
    r#"
== Bytecodes ==
0000 Nil
0001 DefineGlobal        0 '"a"'
0003 Return

"#,
    r#"
== VM Stack Snapshot ==
[]
[nil]
[]

"#
  );
}
#[test]
fn chapter_21_var_init() {
  assert_snapshot!(
    r#"var a = 0;"#,
    r#"
== Bytecodes ==
0000 Constant            1 '0'
0002 DefineGlobal        0 '"a"'
0004 Return

"#,
    r#"
== VM Stack Snapshot ==
[]
[0]
[]

"#
  );
}

#[test]
fn chapter_21() {
  assert_snapshot!(
    r#"
var a = "aaa";
var b = "bbb";
a = "assign add " + b;

print a;
"#,
    r#"
== Bytecodes ==
0000 Constant            1 '"aaa"'
0002 DefineGlobal        0 '"a"'
0004 Constant            3 '"bbb"'
0006 DefineGlobal        2 '"b"'
0008 Constant            5 '"assign add "'
0010 GetGlobal           6 '"b"'
0012 Add
0013 SetGlobal           4 '"a"'
0015 Pop
0016 GetGlobal           7 '"a"'
0018 Print
0019 Return

"#,
    r#"
== VM Stack Snapshot ==
[]
["aaa"]
[]
["bbb"]
[]
["assign add "]
["assign add ", "bbb"]
["assign add bbb"]
["assign add bbb"]
[]
["assign add bbb"]
[]

"#
  );
}
