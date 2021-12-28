use crate::scanner::Scanner;

pub fn compile(source: &str) -> Result<(), String> {
  let mut scanner = Scanner::new(source);
  scanner.print();
  Ok(())
}
