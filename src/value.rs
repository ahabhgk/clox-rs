use std::{cell::RefCell, fmt, rc::Rc};

use crate::{vm::CallFrame, Chunk, VM};

#[derive(Clone)]
pub enum FunctionKind {
  Function { name: String },
  Script,
}

#[derive(Clone)]
pub struct Function {
  pub kind: FunctionKind,
  pub arity: u8,
  pub chunk: Chunk,
}

impl Function {
  pub fn new_function(name: &str) -> Self {
    Self {
      kind: FunctionKind::Function {
        name: name.to_owned(),
      },
      arity: 0,
      chunk: Chunk::new(),
    }
  }

  pub fn new_script() -> Self {
    Self {
      kind: FunctionKind::Script,
      arity: 0,
      chunk: Chunk::new(),
    }
  }

  pub fn call(
    self,
    vm: &mut VM,
    arg_count: u8,
    frame: CallFrame,
  ) -> Result<CallFrame, String> {
    let closure = Closure::new(self, 0);
    closure.call(vm, arg_count, frame)
  }
}

impl fmt::Debug for Function {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let FunctionKind::Function { name } = &self.kind {
      write!(f, "<fun {}>", name)
    } else {
      write!(f, "<script>")
    }
  }
}

#[derive(Clone)]
pub struct Closure {
  pub function: Function,
  pub upvalues_len: u8,
  pub upvalues: Vec<Upvalue>,
}

impl Closure {
  pub fn new(function: Function, upvalues_len: u8) -> Self {
    Self {
      function,
      upvalues_len,
      upvalues: Vec::new(),
    }
  }

  pub fn call(
    self,
    vm: &mut VM,
    arg_count: u8,
    frame: CallFrame,
  ) -> Result<CallFrame, String> {
    if arg_count != self.function.arity {
      return Err(format!(
        "Expected {} arguments but got {}.",
        self.function.arity, arg_count
      ));
    }
    if vm.frames.len() >= u8::MAX.into() {
      return Err("stack overflow.".to_owned());
    }

    vm.frames.push(frame);
    let f_frame = CallFrame::new(self, vm.stack.len() as u8 - arg_count - 1);
    Ok(f_frame)
  }
}

impl fmt::Debug for Closure {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self.function)
  }
}

#[derive(Debug, Clone)]
pub struct Upvalue {
  pub location: *mut Value,
}

impl Upvalue {
  pub fn new(location: &mut Value) -> Self {
    Self { location }
  }

  pub fn get(&self) -> Value {
    unsafe { self.location.as_ref() }.unwrap().clone()
  }

  pub fn set(&mut self, location: &mut Value) {
    self.location = location;
  }
}

#[derive(Clone)]
pub enum Value {
  Bool(bool),
  Nil,
  Number(f64),
  String(String),
  Function(Function),
  Closure(Closure),
}

impl Value {
  pub fn bool(v: bool) -> Self {
    Self::Bool(v)
  }

  pub fn nil() -> Self {
    Self::Nil
  }

  pub fn number(v: f64) -> Self {
    Self::Number(v)
  }

  pub fn string(v: &str) -> Self {
    Self::String(v.to_owned())
  }

  pub fn function(v: Function) -> Self {
    Self::Function(v)
  }

  pub fn closure(v: Closure) -> Self {
    Self::Closure(v)
  }

  pub fn as_bool(self) -> Option<bool> {
    match self {
      Self::Bool(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_number(self) -> Option<f64> {
    match self {
      Self::Number(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_string(self) -> Option<String> {
    match self {
      Self::String(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_function(self) -> Option<Function> {
    match self {
      Self::Function(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_closure(self) -> Option<Closure> {
    match self {
      Self::Closure(v) => Some(v),
      _ => None,
    }
  }

  pub fn is_bool(&self) -> bool {
    matches!(self, Self::Bool(_))
  }

  pub fn is_nil(&self) -> bool {
    matches!(self, Self::Nil)
  }

  pub fn is_number(&self) -> bool {
    matches!(self, Self::Number(_))
  }

  pub fn is_falsey(&self) -> bool {
    self.is_nil() || self.is_bool() && !self.clone().as_bool().unwrap()
  }

  pub fn is_string(&self) -> bool {
    matches!(self, Self::String(_))
  }

  pub fn is_function(&self) -> bool {
    matches!(self, Self::Function(_))
  }

  pub fn equal(a: &Self, b: &Self) -> bool {
    match (a, b) {
      (Self::Number(a), Self::Number(b)) => a == b,
      (Self::Bool(a), Self::Bool(b)) => a == b,
      (Self::Nil, Self::Nil) => true,
      (Self::String(a), Self::String(b)) => a == b,
      _ => false,
    }
  }
}

impl fmt::Debug for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Number(v) => write!(f, "{}", v),
      Self::Bool(v) => write!(f, "{}", v),
      Self::Nil => write!(f, "nil"),
      Self::String(v) => write!(f, "\"{}\"", v),
      Self::Function(v) => write!(f, "{:?}", v),
      Self::Closure(v) => write!(f, "{:?}", v),
    }
  }
}
