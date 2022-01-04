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
  ($source:literal, $message:literal) => {
    fn get_err() -> Result<(), String> {
      let scanner = Scanner::new($source);
      let mut chunk = Chunk::new();
      let mut parser = Parser::new(scanner, &mut chunk);
      parser.advance()?;
      parser.program()?;
      parser.end();
      let mut vm = VM::new(chunk);
      let _ = vm.inspect()?;
      Ok(())
    }
    assert_eq!(get_err().unwrap_err(), $message);
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
fn chapter_21_global_uninit() {
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
fn chapter_21_global_init() {
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

#[test]
fn chapter_22_local() {
  assert_snapshot!(
    r#"
{
  var a = "first";
  var b = "second";
  print a + b;
}
"#,
    r#"
== Bytecodes ==
0000 Constant            0 '"first"'
0002 Constant            1 '"second"'
0004 GetLocal            0
0006 GetLocal            1
0008 Add
0009 Print
0010 Pop
0011 Pop
0012 Return

"#,
    r#"
== VM Stack Snapshot ==
[]
["first"]
["first", "second"]
["first", "second", "first"]
["first", "second", "first", "second"]
["first", "second", "firstsecond"]
["first", "second"]
["first"]
[]

"#
  );
}

#[test]
fn chapter_22_local_uninit() {
  assert_snapshot!(
    r#"
{
  var a = "outer";
  {
    var a = a;
  }
}
"#,
    "Can't read local variable in its own initializer."
  );
}

#[test]
fn chapter_22() {
  assert_snapshot!(
    r#"
{
  var a = 1;
  {
    var b = 2;
    {
      var c = 3;
      {
        var d = 4;
        print a + b + c + d;
      }
      var e = 5;
      print a + e;
    }
  }
  var f = 6;
  {
    var g = 7;
    print f + g;
  }
}
"#,
    r#"
== Bytecodes ==
0000 Constant            0 '1'
0002 Constant            1 '2'
0004 Constant            2 '3'
0006 Constant            3 '4'
0008 GetLocal            0
0010 GetLocal            1
0012 Add
0013 GetLocal            2
0015 Add
0016 GetLocal            3
0018 Add
0019 Print
0020 Pop
0021 Constant            4 '5'
0023 GetLocal            0
0025 GetLocal            3
0027 Add
0028 Print
0029 Pop
0030 Pop
0031 Pop
0032 Constant            5 '6'
0034 Constant            6 '7'
0036 GetLocal            1
0038 GetLocal            2
0040 Add
0041 Print
0042 Pop
0043 Pop
0044 Pop
0045 Return

"#,
    r#"
== VM Stack Snapshot ==
[]
[1]
[1, 2]
[1, 2, 3]
[1, 2, 3, 4]
[1, 2, 3, 4, 1]
[1, 2, 3, 4, 1, 2]
[1, 2, 3, 4, 3]
[1, 2, 3, 4, 3, 3]
[1, 2, 3, 4, 6]
[1, 2, 3, 4, 6, 4]
[1, 2, 3, 4, 10]
[1, 2, 3, 4]
[1, 2, 3]
[1, 2, 3, 5]
[1, 2, 3, 5, 1]
[1, 2, 3, 5, 1, 5]
[1, 2, 3, 5, 6]
[1, 2, 3, 5]
[1, 2, 3]
[1, 2]
[1]
[1, 6]
[1, 6, 7]
[1, 6, 7, 6]
[1, 6, 7, 6, 7]
[1, 6, 7, 13]
[1, 6, 7]
[1, 6]
[1]
[]

"#
  );
}
