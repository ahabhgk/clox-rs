use std::{collections::HashMap, fmt, vec::IntoIter};

use crate::{chunk::Op, parser::compile, value::Value, Chunk};

pub fn interpret(source: &str) -> Result<(), String> {
  let chunk = compile(source)?;
  let mut vm = VM::new(chunk);
  vm.inspect()?;
  Ok(())
}

pub struct VM {
  stack: Vec<Value>,
  codes: IntoIter<usize>,
  constants: Vec<Value>,
  globals: HashMap<String, Value>,
}

impl VM {
  pub fn new(chunk: Chunk) -> Self {
    Self {
      stack: Vec::new(),
      codes: chunk.codes.into_iter(),
      constants: chunk.constants,
      globals: HashMap::new(),
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
        let constant_index = read_code!();
        self.constants.get(constant_index).unwrap().to_owned()
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

    loop {
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
        Op::Pop => {
          pop!();
        }
        Op::GetGlobal => {
          let name = read_constant!();
          let name = name.as_string().unwrap();
          dbg!(name);
          let value =
            self.globals.get(name).ok_or("Undefined variable.")?.clone();
          push!(value);
        }
        Op::DefineGlobal => {
          let name = read_constant!().as_string().unwrap().to_owned();
          self.globals.insert(name, pop!());
        }
        Op::SetGlobal => {
          let name = read_constant!().as_string().unwrap().to_owned();
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
        Op::Return => break,
      };
    }
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
