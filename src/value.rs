use std::fmt;

#[derive(Debug)]
pub enum Value {
  Bool(bool),
  Nil,
  Number(f64),
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

  pub fn as_bool(&self) -> Option<bool> {
    match self {
      Self::Bool(v) => Some(*v),
      _ => None,
    }
  }

  pub fn as_number(&self) -> Option<f64> {
    match self {
      Self::Number(v) => Some(*v),
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
    self.is_nil() || self.is_bool() && !self.as_bool().unwrap()
  }

  pub fn equal(a: &Self, b: &Self) -> bool {
    match (a, b) {
      (Self::Number(a), Self::Number(b)) => a == b,
      (Self::Bool(a), Self::Bool(b)) => a == b,
      (Self::Nil, Self::Nil) => true,
      _ => false,
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Number(v) => write!(f, "{}", v),
      Self::Bool(v) => write!(f, "{}", v),
      Self::Nil => write!(f, "nil"),
    }
  }
}
