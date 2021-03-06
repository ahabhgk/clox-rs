use std::{
  env, fs,
  io::{self, BufRead, Write},
};

use clox_rs::interpret;

fn run_repl() {
  let stdin = io::stdin();
  let stdout = io::stdout();
  let mut reader = stdin.lock();
  let mut writer = stdout.lock();

  loop {
    writer.write_all("> ".as_bytes()).unwrap();
    writer.flush().unwrap();

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    if let Err(e) = interpret(&line) {
      eprintln!("{}", e);
    }
  }
}

fn run_file(path: &str) {
  let source = fs::read_to_string(path).unwrap();

  if let Err(e) = interpret(&source) {
    eprintln!("{}", e);
  }
}

fn main() {
  match env::args().nth(1) {
    Some(path) => run_file(&path),
    None => run_repl(),
  };
}
