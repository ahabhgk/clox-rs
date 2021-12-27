use crate::scanner::Scanner;

pub fn compile(source: &str) -> Result<(), String> {
  let scanner = Scanner::new(source);
  scanner.print();
  Ok(())
}
