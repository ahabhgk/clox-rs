use crate::parser::{ParseFn, Parser};

#[derive(Debug, PartialEq, Eq)]
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

#[derive(PartialEq, PartialOrd)]
pub enum Precedence {
  None,
  Assignment, // =
  Or,         // or
  And,        // and
  Equality,   // == !=
  Comparison, // < > <= >=
  Term,       // + -
  Factor,     // * /
  Unary,      // ! -
  Call,       // . ()
  Primary,
}

#[test]
fn tt() {
  let n = Precedence::None;
  let f = Precedence::Factor;
  let p = Precedence::Primary;
  dbg!(p >= f, f >= n);
}

impl Precedence {
  pub fn up(self) -> Self {
    match self {
      Self::None => Self::Assignment,
      Self::Assignment => Self::Or,
      Self::Or => Self::And,
      Self::And => Self::Equality,
      Self::Equality => Self::Comparison,
      Self::Comparison => Self::Term,
      Self::Term => Self::Factor,
      Self::Factor => Self::Unary,
      Self::Unary => Self::Call,
      Self::Call => Self::Primary,
      Self::Primary => Self::Primary,
    }
  }
}

pub struct Rule<'s, 'c> {
  pub precedence: Precedence,
  pub prefix: Option<ParseFn<'s, 'c>>,
  pub infix: Option<ParseFn<'s, 'c>>,
}

impl<'s, 'c> Rule<'s, 'c> {
  pub fn new(
    precedence: Precedence,
    prefix: Option<ParseFn<'s, 'c>>,
    infix: Option<ParseFn<'s, 'c>>,
  ) -> Self {
    Self {
      precedence,
      prefix,
      infix,
    }
  }
}

impl TokenType {
  pub fn rule<'s, 'c>(&self) -> Rule<'s, 'c> {
    match self {
      Self::LeftParen => {
        Rule::new(Precedence::None, Some(Parser::grouping), None)
      }
      Self::RightParen => Rule::new(Precedence::None, None, None),
      Self::LeftBrace => Rule::new(Precedence::None, None, None),
      Self::RightBrace => Rule::new(Precedence::None, None, None),
      Self::Comma => Rule::new(Precedence::None, None, None),
      Self::Dot => Rule::new(Precedence::None, None, None),
      Self::Minus => {
        Rule::new(Precedence::Term, Some(Parser::unary), Some(Parser::binary))
      }
      Self::Plus => Rule::new(Precedence::Term, None, Some(Parser::binary)),
      Self::Semicolon => Rule::new(Precedence::None, None, None),
      Self::Slash => Rule::new(Precedence::Factor, None, Some(Parser::binary)),
      Self::Star => Rule::new(Precedence::Factor, None, Some(Parser::binary)),
      Self::Bang => Rule::new(Precedence::None, Some(Parser::unary), None),
      Self::BangEqual => {
        Rule::new(Precedence::Equality, None, Some(Parser::binary))
      }
      Self::Equal => Rule::new(Precedence::None, None, None),
      Self::EqualEqual => {
        Rule::new(Precedence::Equality, None, Some(Parser::binary))
      }
      Self::Greater => {
        Rule::new(Precedence::Comparison, None, Some(Parser::binary))
      }
      Self::GreaterEqual => {
        Rule::new(Precedence::Comparison, None, Some(Parser::binary))
      }
      Self::Less => {
        Rule::new(Precedence::Comparison, None, Some(Parser::binary))
      }
      Self::LessEqual => {
        Rule::new(Precedence::Comparison, None, Some(Parser::binary))
      }
      Self::Identifier => Rule::new(Precedence::None, None, None),
      Self::String => Rule::new(Precedence::None, None, None),
      Self::Number => Rule::new(Precedence::None, Some(Parser::number), None),
      Self::And => Rule::new(Precedence::None, None, None),
      Self::Class => Rule::new(Precedence::None, None, None),
      Self::Else => Rule::new(Precedence::None, None, None),
      Self::False => Rule::new(Precedence::None, Some(Parser::literal), None),
      Self::For => Rule::new(Precedence::None, None, None),
      Self::Fun => Rule::new(Precedence::None, None, None),
      Self::If => Rule::new(Precedence::None, None, None),
      Self::Nil => Rule::new(Precedence::None, Some(Parser::literal), None),
      Self::Or => Rule::new(Precedence::None, None, None),
      Self::Print => Rule::new(Precedence::None, None, None),
      Self::Return => Rule::new(Precedence::None, None, None),
      Self::Super => Rule::new(Precedence::None, None, None),
      Self::This => Rule::new(Precedence::None, None, None),
      Self::True => Rule::new(Precedence::None, Some(Parser::literal), None),
      Self::Var => Rule::new(Precedence::None, None, None),
      Self::While => Rule::new(Precedence::None, None, None),
    }
  }
}

#[derive(Debug)]
pub struct Token {
  pub token_type: TokenType,
  start: usize,
  length: usize,
  line: usize,
  pub source: String,
}

impl Token {
  pub fn new(
    token_type: TokenType,
    start: usize,
    length: usize,
    line: usize,
    source: String,
  ) -> Self {
    Self {
      token_type,
      start,
      length,
      line,
      source,
    }
  }
}
