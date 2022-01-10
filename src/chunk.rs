use std::{iter::Enumerate, slice::Iter};

use crate::{
  scope::Upvalue,
  value::{Closure, Function, Value},
};

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
  GetUpvalue,
  SetUpvalue,
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
  Jump,
  JumpIfFalse,
  Loop,
  Call,
  Closure,
  Return,
}

impl From<Op> for u8 {
  fn from(op: Op) -> Self {
    op as u8
  }
}

impl From<u8> for Op {
  fn from(u: u8) -> Self {
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
      10 => Self::GetUpvalue,
      11 => Self::SetUpvalue,
      12 => Self::Equal,
      13 => Self::Greater,
      14 => Self::Less,
      15 => Self::Add,
      16 => Self::Subtract,
      17 => Self::Multiply,
      18 => Self::Divide,
      19 => Self::Not,
      20 => Self::Negate,
      21 => Self::Print,
      22 => Self::Jump,
      23 => Self::JumpIfFalse,
      24 => Self::Loop,
      25 => Self::Call,
      26 => Self::Closure,
      27 => Self::Return,
      _ => unreachable!("{:?}", u),
    }
  }
}

#[derive(Clone)]
pub struct Chunk {
  pub codes: Vec<u8>,
  pub constants: Vec<Value>,
}

impl Chunk {
  pub fn new() -> Self {
    Self {
      codes: Vec::new(),
      constants: Vec::new(),
    }
  }

  pub fn code_len(&self) -> Result<u16, String> {
    let len = self.codes.len();
    if len > u16::MAX.into() {
      return Err("Too much code...".to_owned());
    }
    Ok(len as u16)
  }

  pub fn emit_op(&mut self, op: Op) {
    self.push(op.into())
  }

  pub fn emit_constant(&mut self, constant: Value) -> Result<(), String> {
    let index = self.add_constant(constant)?;
    self.emit_op(Op::Constant);
    self.push(index);
    Ok(())
  }

  pub fn emit_define_global(&mut self, index: u8) {
    self.emit_op(Op::DefineGlobal);
    self.push(index);
  }

  pub fn emit_get_global(&mut self, index: u8) {
    self.emit_op(Op::GetGlobal);
    self.push(index);
  }

  pub fn emit_set_global(&mut self, index: u8) {
    self.emit_op(Op::SetGlobal);
    self.push(index);
  }

  pub fn emit_get_local(&mut self, index: u8) {
    self.emit_op(Op::GetLocal);
    self.push(index);
  }

  pub fn emit_set_local(&mut self, index: u8) {
    self.emit_op(Op::SetLocal);
    self.push(index);
  }

  pub fn emit_get_upvalue(&mut self, index: u8) {
    self.emit_op(Op::GetUpvalue);
    self.push(index);
  }

  pub fn emit_set_upvalue(&mut self, index: u8) {
    self.emit_op(Op::SetUpvalue);
    self.push(index);
  }

  pub fn emit_upvalue(&mut self, upvalue: Upvalue) {
    self.push(if upvalue.is_local { 1 } else { 0 });
    self.push(upvalue.index);
  }

  pub fn emit_jump(&mut self, op: Op) -> Result<u16, String> {
    self.emit_op(op);
    self.push(0xff);
    self.push(0xff);
    let len = self.codes.len();
    if len > u16::MAX.into() {
      return Err("Too much code...".to_owned());
    }
    Ok(len as u16 - 2)
  }

  pub fn patch_jump(&mut self, start: u16) -> Result<(), String> {
    let len = self.codes.len();
    if len > u16::MAX.into() {
      return Err("Too much code to jump over.".to_owned());
    }
    let offset = len as u16 - 2 - start;
    let offset = offset.to_ne_bytes();
    self.write(offset[0], start)?;
    self.write(offset[1], start + 1)?;
    Ok(())
  }

  pub fn emit_loop(&mut self, start: u16) -> Result<(), String> {
    self.emit_op(Op::Loop);
    let len = self.codes.len();
    if len > u16::MAX.into() {
      return Err("Loop body too large.".to_owned());
    }
    let offset = len as u16 + 2 - start;
    let offset = offset.to_ne_bytes();
    self.push(offset[0]);
    self.push(offset[1]);
    Ok(())
  }

  pub fn emit_call(&mut self, arg_count: u8) {
    self.emit_op(Op::Call);
    self.push(arg_count);
  }

  pub fn emit_closure(&mut self, closure: Closure) -> Result<(), String> {
    let index = self.add_constant(Value::closure(closure))?;
    self.emit_op(Op::Closure);
    self.push(index);
    Ok(())
  }

  fn push(&mut self, byte: u8) {
    self.codes.push(byte);
  }

  fn write(&mut self, byte: u8, at: u16) -> Result<(), String> {
    let old = self
      .codes
      .get_mut(at as usize)
      .ok_or("The index is out of bound")?;
    *old = byte;
    Ok(())
  }

  pub fn add_constant(&mut self, constant: Value) -> Result<u8, String> {
    let index = self.constants.len();
    if index > u8::MAX.into() {
      return Err("Too many constants in one chunk.".to_owned());
    }
    self.constants.push(constant);
    Ok(index as u8)
  }

  pub fn debug_bytecodes(&self, prefix: &str) -> String {
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
        Op::GetUpvalue => self.debug_index(&op, &mut codes),
        Op::SetUpvalue => self.debug_index(&op, &mut codes),
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
        Op::Jump => self.debug_jump(&op, index, true, &mut codes),
        Op::JumpIfFalse => self.debug_jump(&op, index, true, &mut codes),
        Op::Loop => self.debug_jump(&op, index, false, &mut codes),
        Op::Call => self.debug_index(&op, &mut codes),
        Op::Closure => {
          let (_, &constant_index) = codes.next().unwrap();
          let constant = self.constants.get(constant_index as usize).unwrap();
          let mut s = format!(
            "{:16} {:4} {:?}\n",
            format!("{:?}", op),
            constant_index,
            constant
          );
          let closure = constant.clone().as_closure().unwrap();
          dbg!(&codes, &closure);
          for _ in 0..closure.upvalues_len {
            let (i, &is_local) = codes.next().unwrap();
            let (_, &upvalue_index) = codes.next().unwrap();
            s.push_str(&format!(
              "{:04} {:21} {} {}\n",
              i,
              "|",
              if is_local == 1 { "local" } else { "upvalue" },
              upvalue_index,
            ));
          }
          s
        }
        Op::Return => self.debug_simple(&op),
      };
      buffer.push_str(&s);
    }

    buffer
  }

  fn debug_simple(&self, op: &Op) -> String {
    format!("{:?}\n", op)
  }

  fn debug_double(&self, op: &Op, codes: &mut Enumerate<Iter<u8>>) -> String {
    let (_, &constant_index) = codes.next().unwrap();
    let constant = self.constants.get(constant_index as usize).unwrap();
    format!(
      "{:16} {:4} '{:?}'\n",
      format!("{:?}", op),
      constant_index,
      constant
    )
  }

  fn debug_index(&self, op: &Op, codes: &mut Enumerate<Iter<u8>>) -> String {
    let (_, &index) = codes.next().unwrap();
    format!("{:16} {:4}\n", format!("{:?}", op), index)
  }

  fn debug_jump(
    &self,
    op: &Op,
    from: usize,
    is_forward: bool,
    codes: &mut Enumerate<Iter<u8>>,
  ) -> String {
    let (_, &offset_0) = codes.next().unwrap();
    let (_, &offset_1) = codes.next().unwrap();
    let offset = unsafe { *[offset_0, offset_1].as_ptr().cast::<u16>() };
    let to = if is_forward {
      from + 3 + offset as usize
    } else {
      from + 3 - offset as usize
    };
    format!("{:16} {:4} -> {}\n", format!("{:?}", op), from, to)
  }
}
