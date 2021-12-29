use std::str::Chars;

pub struct Source<'s> {
  iter: Chars<'s>,
  index: usize,
}

impl<'s> Source<'s> {
  pub fn new(source: &'s str) -> Self {
    Self {
      iter: source.chars(),
      index: 0,
    }
  }

  fn advance(&mut self) -> Option<char> {
    self.index += 1;
    self.current()
  }

  fn current(&self) -> Option<char> {
    debug_assert!(self.index > 0);
    self.get(self.index - 1)
  }

  fn peek(&self) -> Option<char> {
    self.get(self.index)
  }

  fn peek_next(&self) -> Option<char> {
    self.get(self.index + 1)
  }

  fn get(&self, index: usize) -> Option<char> {
    self.iter.clone().nth(index)
  }

  fn slice(&self, start: usize, end: usize) -> &str {
    let str = self.iter.as_str();
    &str[start..end]
  }

  fn test(&mut self, expected: char) -> bool {
    match self.peek() {
      None => false,
      Some(c) if c == expected => {
        self.index += 1;
        true
      }
      _ => false,
    }
  }
}
