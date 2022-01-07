use crate::{Chunk, scope::Scopes};

pub struct Compiler {
  pub chunk: Chunk,
  pub scopes: Scopes,
}

impl Compiler {
  pub fn new() -> Self {
    Self {
      chunk: Chunk::new(),
      scopes: Scopes::new(),
    }
  }
}
