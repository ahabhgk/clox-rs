use std::{
  fmt::{self, Debug},
  iter::Enumerate,
  slice::Iter,
};

use crate::value::Value;

#[derive(Debug)]
pub enum Op {
  Constant = 0,
  Nil,
  True,
  False,
  Pop,
  GetLocal,
  SetLocal,
  GetGlobal,
  DefineGlobal,
  SetGlobal,
  Equal,
  Greater,
  Less,
  Add,
  Subtract,
  Multiply,
  Divide,
  Not,
  Negate,
  Print,
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
      4 => Self::Pop,
      5 => Self::GetLocal,
      6 => Self::SetLocal,
      7 => Self::GetGlobal,
      8 => Self::DefineGlobal,
      9 => Self::SetGlobal,
      10 => Self::Equal,
      11 => Self::Greater,
      12 => Self::Less,
      13 => Self::Add,
      14 => Self::Subtract,
      15 => Self::Multiply,
      16 => Self::Divide,
      17 => Self::Not,
      18 => Self::Negate,
      19 => Self::Print,
      20 => Self::Return,
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

  pub fn emit_define_global(&mut self, index: usize) {
    self.emit_op(Op::DefineGlobal);
    self.write(index);
  }

  pub fn emit_get_global(&mut self, index: usize) {
    self.emit_op(Op::GetGlobal);
    self.write(index);
  }

  pub fn emit_set_global(&mut self, index: usize) {
    self.emit_op(Op::SetGlobal);
    self.write(index);
  }

  pub fn emit_get_local(&mut self, index: usize) {
    self.emit_op(Op::GetLocal);
    self.write(index);
  }

  pub fn emit_set_local(&mut self, index: usize) {
    self.emit_op(Op::SetLocal);
    self.write(index);
  }

  fn write(&mut self, byte: usize) {
    self.codes.push(byte);
  }

  pub fn add_constant(&mut self, constant: Value) -> usize {
    let index = self.constants.len();
    self.constants.push(constant);
    index
  }

  fn debug_bytecodes(&self, prefix: &str) -> String {
    let mut buffer = String::from(format!("{}\n", prefix));

    let mut codes = self.codes.iter().enumerate();

    while let Some((index, &code)) = codes.next() {
      buffer.push_str(&format!("{:04} ", index));

      let op = Op::from(code);
      let s = match op {
        Op::Constant => self.debug_double(&op, &mut codes),
        Op::Nil => self.debug_simple(&op),
        Op::True => self.debug_simple(&op),
        Op::False => self.debug_simple(&op),
        Op::Pop => self.debug_simple(&op),
        Op::GetLocal => self.debug_index(&op, &mut codes),
        Op::SetLocal => self.debug_index(&op, &mut codes),
        Op::GetGlobal => self.debug_double(&op, &mut codes),
        Op::DefineGlobal => self.debug_double(&op, &mut codes),
        Op::SetGlobal => self.debug_double(&op, &mut codes),
        Op::Equal => self.debug_simple(&op),
        Op::Greater => self.debug_simple(&op),
        Op::Less => self.debug_simple(&op),
        Op::Add => self.debug_simple(&op),
        Op::Subtract => self.debug_simple(&op),
        Op::Multiply => self.debug_simple(&op),
        Op::Divide => self.debug_simple(&op),
        Op::Not => self.debug_simple(&op),
        Op::Negate => self.debug_simple(&op),
        Op::Print => self.debug_simple(&op),
        Op::Return => self.debug_simple(&op),
      };
      buffer.push_str(&s);
    }

    buffer
  }

  fn debug_simple(&self, op: &Op) -> String {
    format!("{:?}\n", op)
  }

  fn debug_double(
    &self,
    op: &Op,
    codes: &mut Enumerate<Iter<usize>>,
  ) -> String {
    let (_, &constant_index) = codes.next().unwrap();
    let constant = self.constants.get(constant_index).unwrap();
    format!(
      "{:16} {:4} '{:?}'\n",
      format!("{:?}", op),
      constant_index,
      constant
    )
  }

  fn debug_index(&self, op: &Op, codes: &mut Enumerate<Iter<usize>>) -> String {
    let (_, &index) = codes.next().unwrap();
    format!("{:16} {:4}\n", format!("{:?}", op), index)
  }
}

impl fmt::Debug for Chunk {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.debug_bytecodes("== Bytecodes =="))
  }
}
