use std::str::Chars;

use crate::token::{Token, TokenType};

pub struct Scanner<'source> {
  source: Chars<'source>,
  start: usize,
  current: usize,
  line: usize,
}

impl<'source> Scanner<'source> {
  pub fn new(source: &'source str) -> Self {
    Self {
      source: source.chars(),
      start: 0,
      current: 0,
      line: 1,
    }
  }

  pub fn scan_token(&mut self) -> Token {
    match self.advance() {
      None => self.make_token(TokenType::EOF),
      Some(_) => todo!(),
    }
  }

  fn advance(&mut self) -> Option<char> {
    self.current += 1;
    self.source.clone().nth(self.current)
  }

  fn make_token(&self, token_type: TokenType) -> Token {
    Token::new(token_type, self.start, self.current - self.start, self.line)
  }

  pub fn print(&self) {}
}
