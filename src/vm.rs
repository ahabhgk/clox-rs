use crate::compiler::compile;

pub fn interpret(source: &str) -> Result<(), String> {
  compile(source)?;
  Ok(())
}
