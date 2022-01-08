use std::collections::HashMap;

use crate::{
  chunk::Op,
  parser::compile,
  value::{Function, Value},
  Inspector,
};

pub fn interpret(source: &str) -> Result<(), String> {
  let function = compile(source)?;
  let mut vm = VM::from_function(function);
  vm.run(None)?;
  Ok(())
}

pub struct CallFrame {
  function: Function,
  index: u16,
  start: u8,
}

impl CallFrame {
  pub fn new(function: Function, start: u8) -> Self {
    Self {
      function,
      index: 0,
      start,
    }
  }

  pub fn start(&self) -> u8 {
    self.start
  }

  pub fn step_ahead(&mut self, n: u16) {
    self.index += n;
  }

  pub fn step_back(&mut self, n: u16) {
    self.index -= n;
  }

  pub fn read_byte(&mut self) -> u8 {
    let byte = self.function.chunk.codes.get(self.index as usize).unwrap();
    self.index += 1;
    *byte
  }

  pub fn read_short(&mut self) -> u16 {
    let offset_0 = self.read_byte();
    let offset_1 = self.read_byte();
    unsafe { *[offset_0, offset_1].as_ptr().cast::<u16>() }
  }

  pub fn read_constant(&mut self) -> Value {
    let i = self.read_byte() as usize;
    self.function.chunk.constants.get(i).unwrap().clone()
  }

  pub fn get_local(&mut self, stack: &Vec<Value>) -> Value {
    let index = self.start() + self.read_byte();
    stack.get(index as usize).unwrap().clone()
  }

  pub fn set_local(&mut self, stack: &mut Vec<Value>, value: Value) {
    let index = self.start() + self.read_byte();
    let old = stack.get_mut(index as usize).unwrap();
    *old = value;
  }
}

pub struct VM {
  frames: Vec<CallFrame>,
  stack: Vec<Value>,
  globals: HashMap<String, Value>,
}

impl VM {
  pub fn new() -> Self {
    Self {
      frames: Vec::new(),
      stack: Vec::new(),
      globals: HashMap::new(),
    }
  }

  pub fn from_function(function: Function) -> Self {
    let mut vm = Self::new();
    let f = Value::Function(function.clone());
    let frame = CallFrame::new(function, 0);
    vm.frames.push(frame);
    vm.stack.push(f);
    vm
  }

  fn call(
    &mut self,
    callee: Value,
    arg_count: u8,
    frame: CallFrame,
  ) -> Result<CallFrame, String> {
    match callee {
      Value::Function(f) => {
        if arg_count != f.arity {
          return Err(format!(
            "Expected {} arguments but got {}.",
            f.arity, arg_count
          ));
        }
        if self.frames.len() >= u8::MAX.into() {
          return Err("stack overflow.".to_owned());
        }

        self.frames.push(frame);
        let f_frame = CallFrame::new(f, self.stack.len() as u8 - arg_count - 1);
        Ok(f_frame)
      }
      _ => Err("Can only call functions and classes.".to_owned()),
    }
  }

  fn function_return(&mut self, result: Value, frame: CallFrame) -> CallFrame {
    unsafe { self.stack.set_len(frame.start() as usize) };
    self.stack.push(result);
    self.frames.pop().unwrap()
  }

  pub fn run(
    &mut self,
    mut inspector: Option<Inspector>,
  ) -> Result<Option<Inspector>, String> {
    let mut frame = self.frames.pop().unwrap();
    macro_rules! push {
      ($v:expr) => {
        self.stack.push($v)
      };
    }
    macro_rules! pop {
      () => {
        self.stack.pop().unwrap()
      };
    }
    macro_rules! peek {
      ($distance:expr) => {
        self
          .stack
          .get(self.stack.len() - 1 - $distance as usize)
          .unwrap()
      };
    }

    loop {
      if let Some(ref mut inspector) = inspector {
        inspector.catch_stack(self.stack.clone())
      }

      let code = frame.read_byte();
      let op = Op::from(code);
      match op {
        Op::Constant => {
          let constant = frame.read_constant();
          push!(constant);
        }
        Op::Nil => push!(Value::nil()),
        Op::True => push!(Value::bool(true)),
        Op::False => push!(Value::bool(false)),
        Op::Pop => {
          pop!();
        }
        Op::GetLocal => {
          let value = frame.get_local(&self.stack);
          dbg!(&value);
          push!(value);
        }
        Op::SetLocal => {
          let value = peek!(0).clone();
          frame.set_local(&mut self.stack, value);
        }
        Op::GetGlobal => {
          let name = frame.read_constant();
          let name = name.as_string().unwrap();
          let value =
            self.globals.get(name).ok_or("Undefined variable.")?.clone();
          push!(value);
        }
        Op::DefineGlobal => {
          let name = frame.read_constant().as_string().unwrap().to_owned();
          self.globals.insert(name, pop!());
        }
        Op::SetGlobal => {
          let name = frame.read_constant().as_string().unwrap().to_owned();
          self
            .globals
            .insert(name, peek!(0).clone())
            .ok_or("Undefined variable.")?;
        }
        Op::Equal => {
          let b = pop!();
          let a = pop!();
          push!(Value::bool(Value::equal(&a, &b)));
        }
        Op::Greater => {
          let b = pop!().as_number().ok_or("Operand must be a number.")?;
          let a = pop!().as_number().ok_or("Operand must be a number.")?;
          push!(Value::bool(a > b));
        }
        Op::Less => {
          let b = pop!().as_number().ok_or("Operand must be a number.")?;
          let a = pop!().as_number().ok_or("Operand must be a number.")?;
          push!(Value::bool(a < b));
        }
        Op::Add => {
          let b = pop!();
          let a = pop!();
          if b.is_string() && a.is_string() {
            let b = b.as_string().unwrap();
            let a = a.as_string().unwrap();
            let concat = &format!("{}{}", a, b);
            push!(Value::string(concat));
          } else if b.is_number() && a.is_number() {
            let b = b.as_number().unwrap();
            let a = a.as_number().unwrap();
            push!(Value::number(a + b));
          } else {
            return Err(
              "Operands must be two numbers or two strings.".to_string(),
            );
          }
        }
        Op::Subtract => {
          let b = pop!().as_number().ok_or("Operand must be a number.")?;
          let a = pop!().as_number().ok_or("Operand must be a number.")?;
          push!(Value::number(a - b));
        }
        Op::Multiply => {
          let b = pop!().as_number().ok_or("Operand must be a number.")?;
          let a = pop!().as_number().ok_or("Operand must be a number.")?;
          push!(Value::number(a * b));
        }
        Op::Divide => {
          let b = pop!().as_number().ok_or("Operand must be a number.")?;
          let a = pop!().as_number().ok_or("Operand must be a number.")?;
          push!(Value::number(a / b));
        }
        Op::Not => {
          let v = pop!().is_falsey();
          push!(Value::bool(v));
        }
        Op::Negate => {
          let v = pop!().as_number().ok_or("Operand must be a number.")?;
          push!(Value::number(-v));
        }
        Op::Print => println!("{:?}", pop!()),
        Op::Jump => {
          let jump_offset = frame.read_short();
          frame.step_ahead(jump_offset);
        }
        Op::JumpIfFalse => {
          let jump_offset = frame.read_short();
          if peek!(0).is_falsey() {
            frame.step_ahead(jump_offset);
          }
        }
        Op::Loop => {
          let offset = frame.read_short();
          frame.step_back(offset);
        }
        Op::Call => {
          let arg_count = frame.read_byte();
          let callee = peek!(arg_count).clone();
          frame = self.call(callee, arg_count, frame)?;
        }
        Op::Return => {
          let result = pop!();
          if self.frames.is_empty() {
            pop!();
            break;
          }
          frame = self.function_return(result, frame);
        }
      };
    }
    Ok(inspector)
  }
}
