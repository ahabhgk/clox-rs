use std::{fmt, iter::Enumerate, slice::Iter};

use crate::value::Value;

#[derive(Debug)]
pub enum Op {
  Constant = 0,
  Nil,
  True,
  False,
  Equal,
  Greater,
  Less,
  Add,
  Subtract,
  Multiply,
  Divide,
  Not,
  Negate,
  Return,
}

impl From<Op> for usize {
  fn from(op: Op) -> Self {
    op as usize
  }
}

impl From<usize> for Op {
  fn from(u: usize) -> Self {
    match u {
      0 => Self::Constant,
      1 => Self::Nil,
      2 => Self::True,
      3 => Self::False,
      4 => Self::Equal,
      5 => Self::Greater,
      6 => Self::Less,
      7 => Self::Add,
      8 => Self::Subtract,
      9 => Self::Multiply,
      10 => Self::Divide,
      11 => Self::Not,
      12 => Self::Negate,
      13 => Self::Return,
      _ => unreachable!(),
    }
  }
}

pub struct Chunk {
  pub codes: Vec<usize>,
  pub constants: Vec<Value>,
}

impl Chunk {
  pub fn new() -> Self {
    Self {
      codes: Vec::new(),
      constants: Vec::new(),
    }
  }

  pub fn emit_op(&mut self, op: Op) {
    self.write(op.into())
  }

  pub fn emit_constant(&mut self, constant: Value) {
    let index = self.add_constant(constant);
    self.emit_op(Op::Constant);
    self.write(index);
  }

  fn write(&mut self, byte: usize) {
    self.codes.push(byte);
  }

  fn add_constant(&mut self, constant: Value) -> usize {
    let index = self.constants.len();
    self.constants.push(constant);
    index
  }

  fn debug_bytecodes(&self, prefix: &str) -> String {
    let mut buffer = String::from(format!("{}\n", prefix));

    let mut codes = self.codes.iter().enumerate();
    let mut constants = self.constants.iter();

    while let Some((index, &code)) = codes.next() {
      buffer.push_str(&format!("{:04} ", index));

      let op = Op::from(code);
      let s = match op {
        Op::Constant => self.debug_constant(&op, &mut codes, &mut constants),
        Op::Nil => self.debug_simple(&op),
        Op::True => self.debug_simple(&op),
        Op::False => self.debug_simple(&op),
        Op::Equal => self.debug_simple(&op),
        Op::Greater => self.debug_simple(&op),
        Op::Less => self.debug_simple(&op),
        Op::Add => self.debug_simple(&op),
        Op::Subtract => self.debug_simple(&op),
        Op::Multiply => self.debug_simple(&op),
        Op::Divide => self.debug_simple(&op),
        Op::Not => self.debug_simple(&op),
        Op::Negate => self.debug_simple(&op),
        Op::Return => self.debug_simple(&op),
      };
      buffer.push_str(&s);
    }

    buffer
  }

  fn debug_simple(&self, op: &Op) -> String {
    format!("{:?}\n", op)
  }

  fn debug_constant(
    &self,
    op: &Op,
    codes: &mut Enumerate<Iter<usize>>,
    constants: &mut Iter<Value>,
  ) -> String {
    let (_, &constant_index) = codes.next().unwrap();
    let constant = constants.next().unwrap();
    format!("{:<16?} {:4} '{:?}'\n", op, constant_index, constant)
  }
}

impl fmt::Debug for Chunk {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.debug_bytecodes("== Bytecodes =="))
  }
}
