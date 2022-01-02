use crate::{
  chunk::{Chunk, Op},
  scanner::Scanner,
  token::{Precedence, Token, TokenType},
  value::Value,
};

pub fn compile(source: &str) -> Result<Chunk, String> {
  let scanner = Scanner::new(source);
  let mut chunk = Chunk::new();
  let mut parser = Parser::new(scanner, &mut chunk);
  parser.advance()?; // TODO
  parser.expression()?;
  parser.end();
  Ok(chunk)
}

pub struct Parser<'source, 'chunk> {
  peek: Option<Token>,
  scanner: Scanner<'source>,
  chunk: &'chunk mut Chunk,
}

pub type ParseFn<'s, 'c> = fn(&mut Parser<'s, 'c>, Token) -> Result<(), String>;

impl<'source, 'chunk> Parser<'source, 'chunk> {
  pub fn new(scanner: Scanner<'source>, chunk: &'chunk mut Chunk) -> Self {
    Self {
      peek: None,
      scanner,
      chunk,
    }
  }

  pub fn end(&mut self) {
    self.chunk.emit_op(Op::Return);
  }

  pub fn advance(&mut self) -> Result<Option<Token>, String> {
    let current = self.peek.take();
    self.peek = self.scanner.scan_token()?;
    Ok(current)
  }

  pub fn expression(&mut self) -> Result<(), String> {
    self.parse_precedence(Precedence::Assignment)
  }

  pub fn parse_precedence(
    &mut self,
    precedence: Precedence,
  ) -> Result<(), String> {
    if let Some(token) = self.advance()? {
      let prefix =
        token.token_type.rule().prefix.ok_or("Expect expression.")?;
      prefix(self, token)?;

      while matches!(&self.peek, Some(p) if precedence <= p.token_type.rule().precedence)
      {
        if let Some(token) = self.advance()? {
          let infix =
            token.token_type.rule().infix.ok_or("Expect expression.")?;
          infix(self, token)?;
        }
      }
    }
    Ok(())
  }

  pub fn eat(&mut self, token_type: TokenType) -> Result<(), String> {
    if matches!(&self.peek, Some(p) if p.token_type == token_type) {
      self.advance()?;
      return Ok(());
    }
    Err("TODO".to_owned())
  }

  pub fn grouping(&mut self, _token: Token) -> Result<(), String> {
    self.expression()?;
    self.eat(TokenType::RightParen)
  }

  pub fn number(&mut self, token: Token) -> Result<(), String> {
    let constant =
      token.source.parse::<f64>().map_err(|e| "TODO".to_owned())?;
    self.chunk.emit_constant(Value::number(constant));
    Ok(())
  }

  pub fn string(&mut self, token: Token) -> Result<(), String> {
    let string = &token.source[1..(token.length - 1)];
    let string = string.to_owned();
    self.chunk.emit_constant(Value::string(string));
    Ok(())
  }

  pub fn literal(&mut self, token: Token) -> Result<(), String> {
    match token.token_type {
      TokenType::Nil => self.chunk.emit_op(Op::Nil),
      TokenType::False => self.chunk.emit_op(Op::False),
      TokenType::True => self.chunk.emit_op(Op::True),
      _ => unreachable!(),
    }
    Ok(())
  }

  pub fn unary(&mut self, token: Token) -> Result<(), String> {
    self.parse_precedence(Precedence::Unary)?;

    match token.token_type {
      TokenType::Bang => self.chunk.emit_op(Op::Not),
      TokenType::Minus => self.chunk.emit_op(Op::Negate),
      _ => unreachable!(),
    }
    Ok(())
  }

  pub fn binary(&mut self, token: Token) -> Result<(), String> {
    let precedence = token.token_type.rule().precedence;
    self.parse_precedence(precedence.up())?;

    match token.token_type {
      TokenType::BangEqual => {
        self.chunk.emit_op(Op::Equal);
        self.chunk.emit_op(Op::Not);
      }
      TokenType::EqualEqual => self.chunk.emit_op(Op::Equal),
      TokenType::Greater => self.chunk.emit_op(Op::Greater),
      TokenType::GreaterEqual => {
        self.chunk.emit_op(Op::Less);
        self.chunk.emit_op(Op::Not);
      }
      TokenType::Less => self.chunk.emit_op(Op::Less),
      TokenType::LessEqual => {
        self.chunk.emit_op(Op::Greater);
        self.chunk.emit_op(Op::Not);
      }
      TokenType::Plus => self.chunk.emit_op(Op::Add),
      TokenType::Minus => self.chunk.emit_op(Op::Subtract),
      TokenType::Star => self.chunk.emit_op(Op::Multiply),
      TokenType::Slash => self.chunk.emit_op(Op::Divide),
      _ => unreachable!(),
    }
    Ok(())
  }
}
