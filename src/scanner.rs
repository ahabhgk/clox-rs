use std::str::Chars;

use crate::token::{Token, TokenType};

fn is_alpha(c: char) -> bool {
  c.is_ascii_alphabetic()
}

fn is_digit(c: char) -> bool {
  c.is_ascii_digit()
}

pub struct Scanner<'source> {
  source: Chars<'source>,
  start: usize,
  index: usize,
  line: usize,
}

impl<'source> Scanner<'source> {
  pub fn new(source: &'source str) -> Self {
    Self {
      source: source.chars(),
      start: 0,
      index: 0,
      line: 1,
    }
  }

  pub fn scan_token(&mut self) -> Result<Option<Token>, String> {
    self.skip_whitespace();
    self.start = self.index;

    let t = match self.advance() {
      None => return Ok(None),
      Some(c) => match c {
        '(' => self.make_token(TokenType::LeftParen),
        ')' => self.make_token(TokenType::RightParen),
        '{' => self.make_token(TokenType::LeftBrace),
        '}' => self.make_token(TokenType::RightBrace),
        ';' => self.make_token(TokenType::Semicolon),
        ',' => self.make_token(TokenType::Comma),
        '.' => self.make_token(TokenType::Dot),
        '-' => self.make_token(TokenType::Minus),
        '+' => self.make_token(TokenType::Plus),
        '/' => self.make_token(TokenType::Slash),
        '*' => self.make_token(TokenType::Star),
        '!' => {
          if self.test('=') {
            self.make_token(TokenType::BangEqual)
          } else {
            self.make_token(TokenType::Bang)
          }
        }
        '=' => {
          if self.test('=') {
            self.make_token(TokenType::EqualEqual)
          } else {
            self.make_token(TokenType::Equal)
          }
        }
        '<' => {
          if self.test('=') {
            self.make_token(TokenType::LessEqual)
          } else {
            self.make_token(TokenType::Less)
          }
        }
        '>' => {
          if self.test('=') {
            self.make_token(TokenType::GreaterEqual)
          } else {
            self.make_token(TokenType::Greater)
          }
        }
        '"' => self.scan_string()?,
        _ if is_alpha(c) => self.scan_keyword_or_identifier(),
        _ if is_digit(c) => self.scan_number(),
        _ => return Err("Unexpected character.".to_string()),
      },
    };
    Ok(Some(t))
  }

  fn skip_whitespace(&mut self) {
    while let Some(c) = self.peek() {
      match c {
        ' ' | '\r' | '\t' => {
          self.advance();
        }
        '\n' => {
          self.line += 1;
          self.advance();
        }
        '/' => {
          if let Some('/') = self.peek_next() {
            while matches!(self.peek(), Some(c) if c != '\n') {
              self.advance();
            }
          } else {
            return;
          }
        }
        _ => return,
      };
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
    self.source.clone().nth(index)
  }

  fn slice(&self, start: usize, end: usize) -> &str {
    let str = self.source.as_str();
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

  fn make_token(&self, token_type: TokenType) -> Token {
    Token::new(
      token_type,
      self.start,
      self.index - self.start,
      self.line,
      self.slice(self.start, self.index).to_owned(),
    )
  }

  fn scan_string(&mut self) -> Result<Token, String> {
    loop {
      match self.peek() {
        None => return Err("Unterminated string.".to_string()),
        Some('"') => {
          self.advance();
          break;
        }
        Some('\n') => {
          self.line += 1;
          self.advance();
        }
        _ => {
          self.advance();
        }
      }
    }
    Ok(self.make_token(TokenType::String))
  }

  fn scan_number(&mut self) -> Token {
    while matches!(self.peek(), Some(c) if is_digit(c)) {
      self.advance();
    }
    if matches!(self.peek(), Some('.'))
      && matches!(self.peek_next(), Some(c) if is_digit(c))
    {
      self.advance();

      while matches!(self.peek(), Some(c) if is_digit(c)) {
        self.advance();
      }
    }
    self.make_token(TokenType::Number)
  }

  fn scan_keyword_or_identifier(&mut self) -> Token {
    while matches!(self.peek(), Some(c) if is_alpha(c) || is_digit(c)) {
      self.advance();
    }
    self.make_token(self.keyword_or_identifier_type())
  }

  fn keyword_or_identifier_type(&self) -> TokenType {
    match self.get(self.start).unwrap() {
      'a' => self.check_keyword(1, "nd", TokenType::And),
      'c' => self.check_keyword(1, "lass", TokenType::Class),
      'e' => self.check_keyword(1, "lse", TokenType::Else),
      'i' => self.check_keyword(1, "f", TokenType::If),
      'n' => self.check_keyword(1, "il", TokenType::Nil),
      'o' => self.check_keyword(1, "r", TokenType::Or),
      'p' => self.check_keyword(1, "rint", TokenType::Print),
      'r' => self.check_keyword(1, "eturn", TokenType::Return),
      's' => self.check_keyword(1, "uper", TokenType::Super),
      'v' => self.check_keyword(1, "ar", TokenType::Var),
      'w' => self.check_keyword(1, "hile", TokenType::While),
      'f' => match self.get(self.start + 1) {
        Some('a') => self.check_keyword(2, "lse", TokenType::False),
        Some('o') => self.check_keyword(2, "r", TokenType::For),
        Some('u') => self.check_keyword(2, "n", TokenType::Fun),
        _ => TokenType::Identifier,
      },
      't' => match self.get(self.start + 1) {
        Some('h') => self.check_keyword(2, "is", TokenType::This),
        Some('r') => self.check_keyword(2, "ue", TokenType::True),
        _ => TokenType::Identifier,
      },
      _ => TokenType::Identifier,
    }
  }

  fn check_keyword(
    &self,
    start: usize,
    rest: &str,
    token_type: TokenType,
  ) -> TokenType {
    let len = rest.len();
    if self.index - self.start == start + len
      && self.slice(self.start + start, self.start + start + len) == rest
    {
      return token_type;
    }
    TokenType::Identifier
  }

  pub fn print(&mut self) {
    while let Ok(Some(t)) = self.scan_token() {
      println!("{:?}", t);
    }
  }
}
