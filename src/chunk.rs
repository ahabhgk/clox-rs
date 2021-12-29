use std::fmt::Debug;

#[derive(Debug)]
pub enum Op {
  Constant = 0,
  Add,
  Subtract,
  Multiply,
  Divide,
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
      1 => Self::Add,
      2 => Self::Subtract,
      3 => Self::Multiply,
      4 => Self::Divide,
      5 => Self::Negate,
      6 => Self::Return,
      _ => unreachable!(),
    }
  }
}

pub struct Chunk {
  pub codes: Vec<usize>,
  pub constants: Vec<f64>,
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

  pub fn emit_constant(&mut self, constant: f64) {
    let index = self.add_constant(constant);
    self.emit_op(Op::Constant);
    self.write(index);
  }

  fn write(&mut self, byte: usize) {
    self.codes.push(byte);
  }

  fn add_constant(&mut self, constant: f64) -> usize {
    let index = self.constants.len();
    self.constants.push(constant);
    index
  }

  pub fn print(&self) {
    println!("== Ops ==");

    let mut offset = 0;
    while offset < self.codes.len() {
      print!("{:04} ", offset);
      let code = *self.codes.get(offset).unwrap();
      let op = Op::from(code);
      offset = match op {
        Op::Constant => self.print_constant(&op, offset),
        Op::Add => self.print_simple(&op, offset),
        Op::Subtract => self.print_simple(&op, offset),
        Op::Multiply => self.print_simple(&op, offset),
        Op::Divide => self.print_simple(&op, offset),
        Op::Negate => self.print_simple(&op, offset),
        Op::Return => self.print_simple(&op, offset),
      };
    }
  }

  fn print_simple(&self, op: &Op, offset: usize) -> usize {
    println!("{:?}", op);
    offset + 1
  }

  fn print_constant(&self, op: &Op, offset: usize) -> usize {
    let index = *self.codes.get(offset + 1).unwrap();
    let constant = *self.constants.get(index).unwrap();
    println!("{:<16?} {:4} '{}'", op, index, constant);
    offset + 2
  }
}
