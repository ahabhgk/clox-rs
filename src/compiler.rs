use crate::{
  scope::{Scopes, Upvalue},
  value::Function,
  Chunk,
};

pub struct Compiler {
  enclosing: Option<Box<Compiler>>,
  pub function: Function,
  pub scopes: Scopes,
  pub upvalues: Vec<Upvalue>,
}

impl Compiler {
  pub fn script() -> Self {
    Self {
      enclosing: None,
      function: Function::new_script(),
      scopes: Scopes::new(),
      upvalues: Vec::new(),
    }
  }

  pub fn function(self, name: &str) -> Self {
    Self {
      enclosing: Some(Box::new(self)),
      function: Function::new_function(name),
      scopes: Scopes::new(),
      upvalues: Vec::new(),
    }
  }

  pub fn end(self) -> (Option<Compiler>, Function, Vec<Upvalue>) {
    let function = self.function;
    let enclosing = self.enclosing.map(|c| *c);
    (enclosing, function, self.upvalues)
  }

  pub fn chunk(&mut self) -> &mut Chunk {
    &mut self.function.chunk
  }

  pub fn resolve_upvalue(&mut self, name: &str) -> Result<Option<u8>, String> {
    if let Some(enclosing) = &mut self.enclosing {
      if let Some(local) = enclosing.scopes.resolve_local(name)? {
        local.is_captured = true;
        let index = local.index;
        return Ok(Some(self.add_upvalue(index, true)?));
      } else {
        if let Some(index) = enclosing.resolve_upvalue(name)? {
          return Ok(Some(self.add_upvalue(index, false)?));
        }
      }
    }
    Ok(None)
  }

  fn add_upvalue(&mut self, index: u8, is_local: bool) -> Result<u8, String> {
    let len = self.upvalues.len();
    if len > u8::MAX.into() {
      return Err("Too many closure variables in function.".to_owned());
    }
    self.upvalues.push(Upvalue { index, is_local });
    Ok(len as u8)
  }

  pub fn emit_upvalues(&mut self, upvalues: Vec<Upvalue>) {
    for upvalue in upvalues {
      self.chunk().emit_upvalue(upvalue)
    }
  }
}
