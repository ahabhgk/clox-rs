use std::collections::HashMap;

pub struct Scopes {
  scopes: Vec<Scope>,
  count: u8,
}

impl Scopes {
  pub fn new() -> Self {
    Self {
      scopes: Vec::new(),
      count: 1, // CallFrame slot zero
    }
  }

  pub fn push(&mut self) {
    self.scopes.push(Scope::new());
  }

  pub fn pop(&mut self) -> Option<Scope> {
    self.scopes.pop().map(|scope| {
      self.count -= scope.len() as u8;
      scope
    })
  }

  pub fn is_empty(&self) -> bool {
    self.scopes.is_empty()
  }

  pub fn current_has(&mut self, name: &str) -> Option<bool> {
    self.scopes.last().map(|scope| scope.has(name))
  }

  pub fn define_uninit_local(&mut self, name: String) -> Result<(), String> {
    let index = self.count;
    let scope = self
      .scopes
      .last_mut()
      .ok_or("Can't define a local variable without scope.")?;
    scope.define(name, index);
    self.count = self
      .count
      .checked_add(1)
      .ok_or("Too many local variables in function.")?;
    Ok(())
  }

  pub fn mark_init_local(&mut self, name: &str) {
    for scope in self.scopes.iter_mut().rev() {
      if let Some(local) = scope.get_mut(name) {
        local.mark_init();
        return;
      }
    }
  }

  pub fn resolve_local(&self, name: &str) -> Result<Option<&Local>, String> {
    for scope in self.scopes.iter().rev() {
      if let Some(local) = scope.get(name) {
        if !local.is_init {
          return Err(
            "Can't read local variable in its own initializer.".to_owned(),
          );
        }
        return Ok(Some(local));
      }
    }
    Ok(None)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Local {
  pub is_init: bool,
  pub index: u8,
}

impl Local {
  pub fn new_uninit(index: u8) -> Self {
    Self {
      is_init: false,
      index,
    }
  }

  pub fn mark_init(&mut self) {
    self.is_init = true;
  }
}

pub struct Scope {
  locals: HashMap<String, Local>,
}

impl Scope {
  pub fn new() -> Self {
    Self {
      locals: HashMap::new(),
    }
  }

  pub fn has(&self, name: &str) -> bool {
    self.locals.contains_key(name)
  }

  pub fn define(&mut self, name: String, index: u8) {
    let local = Local::new_uninit(index);
    self.locals.insert(name, local);
  }

  pub fn get(&self, name: &str) -> Option<&Local> {
    self.locals.get(name)
  }

  pub fn get_mut(&mut self, name: &str) -> Option<&mut Local> {
    self.locals.get_mut(name)
  }

  pub fn len(&self) -> usize {
    self.locals.len()
  }
}
