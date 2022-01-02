use std::fmt;

use crate::{chunk::Op, parser::compile, value::Value};

pub fn interpret(source: &str) -> Result<(), String> {
  let chunk = compile(source)?;
  let mut vm = VM::new(chunk.codes.into_iter(), chunk.constants.into_iter());
  vm.inspect()?;
  Ok(())
}

pub struct VM<T: Iterator<Item = usize>, U: Iterator<Item = Value>> {
  stack: Vec<Value>,
  codes: T,
  constants: U,
}

impl<T: Iterator<Item = usize>, U: Iterator<Item = Value>> VM<T, U> {
  pub fn new(codes: T, constants: U) -> Self {
    Self {
      stack: Vec::new(),
      codes,
      constants,
    }
  }

  pub fn inspect(&mut self) -> Result<Inspector, String> {
    macro_rules! read_code {
      () => {
        self.codes.next().unwrap()
      };
    }
    macro_rules! read_constant {
      () => {{
        read_code!();
        self.constants.next().unwrap()
      }};
    }
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
        self.stack.get(self.stack.len() - 1 - $distance).unwrap()
      };
    }

    let mut inspector = Inspector {
      stack_snapshot: Vec::new(),
    };

    let _result = loop {
      inspector.stack_snapshot.push(self.stack.clone());

      let code = read_code!();
      let op = Op::from(code);
      match op {
        Op::Constant => {
          let constant = read_constant!();
          push!(constant);
        }
        Op::Nil => push!(Value::nil()),
        Op::True => push!(Value::bool(true)),
        Op::False => push!(Value::bool(false)),
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
            let concat = format!("{}{}", a, b);
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
        Op::Return => break pop!(),
      }
    };
    Ok(inspector)
  }
}

pub struct Inspector {
  stack_snapshot: Vec<Vec<Value>>,
}

impl fmt::Debug for Inspector {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "== VM Stack Snapshot ==")?;
    for snapshot in &self.stack_snapshot {
      writeln!(f, "{:?}", snapshot)?;
    }
    Ok(())
  }
}
