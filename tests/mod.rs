use std::fmt;

use clox_rs::{Parser, Scanner, VM};
use expect_test::{expect, Expect};

fn check(actual: &impl fmt::Debug, expect: Expect) {
  expect.assert_debug_eq(actual);
}

macro_rules! assert_snapshot {
  ($source:literal, $bytecodes:literal, $stack_snapshot:literal) => {
    let scanner = Scanner::new($source);
    let mut parser = Parser::new(scanner);
    parser.advance().unwrap();
    parser.program().unwrap();
    let f = parser.end_compiler();
    check(&f.chunk, expect![[$bytecodes]]);
    let mut vm = VM::from_function(f);
    let inspector = vm.inspect().unwrap();
    check(&inspector, expect![[$stack_snapshot]])
  };
  ($source:literal, $message:literal) => {
    fn get_err() -> Result<(), String> {
      let scanner = Scanner::new($source);
      let mut parser = Parser::new(scanner);
      parser.advance()?;
      parser.program()?;
      let f = parser.end_compiler();
      let mut vm = VM::from_function(f);
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
[<script>]
[<script>, 1]
[<script>, -1]
[<script>, -1, 2]
[<script>, 1]
[<script>, 1, 3]
[<script>, 3]
[<script>, 3, 4]
[<script>, 3, -4]
[<script>, 7]
[<script>]

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
[<script>]
[<script>, 5]
[<script>, 5, 4]
[<script>, 1]
[<script>, 1, 3]
[<script>, 1, 3, 2]
[<script>, 1, 6]
[<script>, false]
[<script>, false, nil]
[<script>, false, true]
[<script>, false]
[<script>, true]
[<script>]

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
[<script>]
[<script>, "aha"]
[<script>, "aha", "b"]
[<script>, "ahab"]
[<script>]

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
[<script>]
[<script>, 1]
[<script>, 1, 2]
[<script>, 3]
[<script>]
[<script>, 3]
[<script>, 3, 4]
[<script>, 12]
[<script>]

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
[<script>]
[<script>, nil]
[<script>]

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
[<script>]
[<script>, 0]
[<script>]

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
[<script>]
[<script>, "aaa"]
[<script>]
[<script>, "bbb"]
[<script>]
[<script>, "assign add "]
[<script>, "assign add ", "bbb"]
[<script>, "assign add bbb"]
[<script>, "assign add bbb"]
[<script>]
[<script>, "assign add bbb"]
[<script>]

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
[<script>]
[<script>, "first"]
[<script>, "first", "second"]
[<script>, "first", "second", "first"]
[<script>, "first", "second", "first", "second"]
[<script>, "first", "second", "firstsecond"]
[<script>, "first", "second"]
[<script>, "first"]
[<script>]

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
[<script>]
[<script>, 1]
[<script>, 1, 2]
[<script>, 1, 2, 3]
[<script>, 1, 2, 3, 4]
[<script>, 1, 2, 3, 4, 1]
[<script>, 1, 2, 3, 4, 1, 2]
[<script>, 1, 2, 3, 4, 3]
[<script>, 1, 2, 3, 4, 3, 3]
[<script>, 1, 2, 3, 4, 6]
[<script>, 1, 2, 3, 4, 6, 4]
[<script>, 1, 2, 3, 4, 10]
[<script>, 1, 2, 3, 4]
[<script>, 1, 2, 3]
[<script>, 1, 2, 3, 5]
[<script>, 1, 2, 3, 5, 1]
[<script>, 1, 2, 3, 5, 1, 5]
[<script>, 1, 2, 3, 5, 6]
[<script>, 1, 2, 3, 5]
[<script>, 1, 2, 3]
[<script>, 1, 2]
[<script>, 1]
[<script>, 1, 6]
[<script>, 1, 6, 7]
[<script>, 1, 6, 7, 6]
[<script>, 1, 6, 7, 6, 7]
[<script>, 1, 6, 7, 13]
[<script>, 1, 6, 7]
[<script>, 1, 6]
[<script>, 1]
[<script>]

"#
  );
}

#[test]
fn chapter_23_if_else() {
  assert_snapshot!(
    r#"if (true) print "yes"; else print "no";"#,
    r#"
== Bytecodes ==
0000 True
0001 JumpIfFalse         1 -> 11
0004 Pop
0005 Constant            0 '"yes"'
0007 Print
0008 Jump                8 -> 15
0011 Pop
0012 Constant            1 '"no"'
0014 Print
0015 Return

"#,
    r#"
== VM Stack Snapshot ==
[<script>]
[<script>, true]
[<script>, true]
[<script>]
[<script>, "yes"]
[<script>]
[<script>]

"#
  );
}

#[test]
fn chapter_23_and_or() {
  assert_snapshot!(
    r#"
nil and "bad";
1 or true;
"#,
    r#"
== Bytecodes ==
0000 Nil
0001 JumpIfFalse         1 -> 7
0004 Pop
0005 Constant            0 '"bad"'
0007 Pop
0008 Constant            1 '1'
0010 JumpIfFalse        10 -> 16
0013 Jump               13 -> 18
0016 Pop
0017 True
0018 Pop
0019 Return

"#,
    r#"
== VM Stack Snapshot ==
[<script>]
[<script>, nil]
[<script>, nil]
[<script>]
[<script>, 1]
[<script>, 1]
[<script>, 1]
[<script>]

"#
  );
}

#[test]
fn chapter_23_while() {
  assert_snapshot!(
    r#"
var a = 0;
while (a < 3) {
  a = a + 1;
}
"#,
    r#"
== Bytecodes ==
0000 Constant            1 '0'
0002 DefineGlobal        0 '"a"'
0004 GetGlobal           2 '"a"'
0006 Constant            3 '3'
0008 Less
0009 JumpIfFalse         9 -> 24
0012 Pop
0013 GetGlobal           5 '"a"'
0015 Constant            6 '1'
0017 Add
0018 SetGlobal           4 '"a"'
0020 Pop
0021 Loop               21 -> 4
0024 Pop
0025 Return

"#,
    r#"
== VM Stack Snapshot ==
[<script>]
[<script>, 0]
[<script>]
[<script>, 0]
[<script>, 0, 3]
[<script>, true]
[<script>, true]
[<script>]
[<script>, 0]
[<script>, 0, 1]
[<script>, 1]
[<script>, 1]
[<script>]
[<script>]
[<script>, 1]
[<script>, 1, 3]
[<script>, true]
[<script>, true]
[<script>]
[<script>, 1]
[<script>, 1, 1]
[<script>, 2]
[<script>, 2]
[<script>]
[<script>]
[<script>, 2]
[<script>, 2, 3]
[<script>, true]
[<script>, true]
[<script>]
[<script>, 2]
[<script>, 2, 1]
[<script>, 3]
[<script>, 3]
[<script>]
[<script>]
[<script>, 3]
[<script>, 3, 3]
[<script>, false]
[<script>, false]
[<script>]

"#
  );
}

#[test]
fn chapter_23_for() {
  assert_snapshot!(
    r#"for (var a = 0; a < 3; a = a + 1) print a;"#,
    r#"
== Bytecodes ==
0000 Constant            0 '0'
0002 GetLocal            0
0004 Constant            1 '3'
0006 Less
0007 JumpIfFalse         7 -> 31
0010 Pop
0011 Jump               11 -> 25
0014 GetLocal            0
0016 Constant            2 '1'
0018 Add
0019 SetLocal            0
0021 Pop
0022 Loop               22 -> 2
0025 GetLocal            0
0027 Print
0028 Loop               28 -> 14
0031 Pop
0032 Pop
0033 Return

"#,
    r#"
== VM Stack Snapshot ==
[<script>]
[<script>, 0]
[<script>, 0, 0]
[<script>, 0, 0, 3]
[<script>, 0, true]
[<script>, 0, true]
[<script>, 0]
[<script>, 0]
[<script>, 0, 0]
[<script>, 0]
[<script>, 0]
[<script>, 0, 0]
[<script>, 0, 0, 1]
[<script>, 0, 1]
[<script>, 1, 1]
[<script>, 1]
[<script>, 1]
[<script>, 1, 1]
[<script>, 1, 1, 3]
[<script>, 1, true]
[<script>, 1, true]
[<script>, 1]
[<script>, 1]
[<script>, 1, 1]
[<script>, 1]
[<script>, 1]
[<script>, 1, 1]
[<script>, 1, 1, 1]
[<script>, 1, 2]
[<script>, 2, 2]
[<script>, 2]
[<script>, 2]
[<script>, 2, 2]
[<script>, 2, 2, 3]
[<script>, 2, true]
[<script>, 2, true]
[<script>, 2]
[<script>, 2]
[<script>, 2, 2]
[<script>, 2]
[<script>, 2]
[<script>, 2, 2]
[<script>, 2, 2, 1]
[<script>, 2, 3]
[<script>, 3, 3]
[<script>, 3]
[<script>, 3]
[<script>, 3, 3]
[<script>, 3, 3, 3]
[<script>, 3, false]
[<script>, 3, false]
[<script>, 3]
[<script>]

"#
  );
}
