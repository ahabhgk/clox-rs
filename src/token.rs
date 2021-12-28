#[derive(Debug)]
pub enum TokenType {
  // Single-character tokens.
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Slash,
  Star,
  // One or two character tokens.
  Bang,
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  // Literals.
  Identifier,
  String,
  Number,
  // Keywords.
  And,
  Class,
  Else,
  False,
  For,
  Fun,
  If,
  Nil,
  Or,
  Print,
  Return,
  Super,
  This,
  True,
  Var,
  While,
}

#[derive(Debug)]
pub struct Token {
  token_type: TokenType,
  start: usize,
  length: usize,
  line: usize,
}

impl Token {
  pub fn new(
    token_type: TokenType,
    start: usize,
    length: usize,
    line: usize,
  ) -> Self {
    Self {
      token_type,
      start,
      length,
      line,
    }
  }
}
