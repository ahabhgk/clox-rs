use std::fmt;

use crate::value::{Function, FunctionKind, Value};

pub struct Inspector {
  bytecode_snapshot: Vec<Function>,
  stack_snapshot: Vec<Vec<Value>>,
}

pub struct BytecodeSnapshot(Vec<Function>);

pub struct StackSnapshot(Vec<Vec<Value>>);

impl fmt::Debug for BytecodeSnapshot {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for fun in &self.0 {
      let name = if let FunctionKind::Function { name } = &fun.kind {
        format!("<fun {}>", name)
      } else {
        "<script>".to_owned()
      };
      let s = fun.chunk.debug_bytecodes(&format!("== {} ==", &name));
      write!(f, "{}", s)?;
    }
    Ok(())
  }
}

impl fmt::Debug for StackSnapshot {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "== VM Stack Snapshot ==")?;
    for snapshot in &self.0 {
      writeln!(f, "{:?}", snapshot)?;
    }
    Ok(())
  }
}

impl Inspector {
  pub fn new() -> Self {
    Self {
      bytecode_snapshot: Vec::new(),
      stack_snapshot: Vec::new(),
    }
  }

  pub fn catch_bytecode(&mut self, f: Function) {
    self.bytecode_snapshot.push(f);
  }

  pub fn catch_stack(&mut self, s: Vec<Value>) {
    self.stack_snapshot.push(s);
  }

  pub fn debug_bytecode(&self) -> BytecodeSnapshot {
    BytecodeSnapshot(self.bytecode_snapshot.clone())
  }

  pub fn debug_stack(&self) -> StackSnapshot {
    StackSnapshot(self.stack_snapshot.clone())
  }
}
