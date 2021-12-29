use crate::{
  chunk::{Chunk, Op},
  parser::compile,
};

pub fn interpret(source: &str) -> Result<(), String> {
  let mut chunk = compile(source)?;
  let mut vm = VM::new(&mut chunk);
  vm.debug_run();
  Ok(())
}

pub struct VM<'chunk> {
  stack: Vec<f64>,
  chunk: &'chunk mut Chunk,
}

impl<'chunk> VM<'chunk> {
  pub fn new(chunk: &'chunk mut Chunk) -> Self {
    Self {
      stack: Vec::new(),
      chunk,
    }
  }

  pub fn debug_run(&mut self) {
    let mut offset = 0;

    macro_rules! read_code {
      () => {{
        let code = *self.chunk.codes.get(offset).unwrap();
        offset += 1;
        code
      }};
    }
    macro_rules! read_constant {
      () => {
        *self.chunk.constants.get(read_code!()).unwrap()
      };
    }
    macro_rules! push {
      ($v:expr) => {
        self.stack.push($v);
      };
    }
    macro_rules! pop {
      () => {
        self.stack.pop().unwrap()
      };
    }

    loop {
      println!("{:?}", self.stack);
      let code = read_code!();
      let op = Op::from(code);
      match op {
        Op::Constant => {
          let constant = read_constant!();
          push!(constant);
        }
        Op::Add => {
          let b = pop!();
          let a = pop!();
          push!(a + b);
        }
        Op::Subtract => {
          let b = pop!();
          let a = pop!();
          push!(a - b);
        }
        Op::Multiply => {
          let b = pop!();
          let a = pop!();
          push!(a * b);
        }
        Op::Divide => {
          let b = pop!();
          let a = pop!();
          push!(a / b);
        }
        Op::Negate => {
          let v = pop!();
          push!(-v);
        }
        Op::Return => {
          let v = pop!();
          println!("{}", v);
          return;
        }
      }
    }
  }
}
