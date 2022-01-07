use crate::{scope::Scopes, value::Function, Chunk};

pub struct Compiler {
  enclosing: Option<Box<Compiler>>,
  pub function: Function,
  pub scopes: Scopes,
}

impl Compiler {
  pub fn new() -> Self {
    Self {
      enclosing: None,
      function: Function::new_script(),
      scopes: Scopes::new(),
    }
  }

  pub fn extend(self) -> Self {
    Self {
      enclosing: Some(Box::new(self)),
      function: Function::new_function("todo".to_owned()),
      scopes: Scopes::new(),
    }
  }

  pub fn end(self) -> (Option<Compiler>, Function) {
    let function = self.function;
    let compiler = self.enclosing.map(|c| *c);
    (compiler, function)
  }

  pub fn chunk(&mut self) -> &mut Chunk {
    &mut self.function.chunk
  }
}
