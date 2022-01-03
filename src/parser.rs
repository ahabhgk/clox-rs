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
  parser.program()?;
  parser.end();
  Ok(chunk)
}

pub struct Parser<'source, 'chunk> {
  peek: Option<Token>,
  scanner: Scanner<'source>,
  chunk: &'chunk mut Chunk,
}

pub type ParseFn<'s, 'c> =
  fn(&mut Parser<'s, 'c>, Token, bool) -> Result<(), String>;

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

  pub fn eat(
    &mut self,
    token_type: TokenType,
    message: &str,
  ) -> Result<Token, String> {
    if matches!(&self.peek, Some(p) if p.token_type == token_type) {
      let token = self.advance()?.unwrap();
      return Ok(token);
    }
    Err(message.to_owned())
  }

  pub fn match_token(&mut self, token_type: TokenType) -> bool {
    self.eat(token_type, "").is_ok()
  }

  pub fn expression(&mut self) -> Result<(), String> {
    self.parse_precedence(Precedence::Assignment)
  }

  pub fn print_statement(&mut self) -> Result<(), String> {
    self.expression()?;
    self.eat(TokenType::Semicolon, "Expect ';' after value.")?;
    self.chunk.emit_op(Op::Print);
    Ok(())
  }

  pub fn expression_statement(&mut self) -> Result<(), String> {
    self.expression()?;
    self.eat(TokenType::Semicolon, "Expect ';' after expression.")?;
    self.chunk.emit_op(Op::Pop);
    Ok(())
  }

  pub fn statement(&mut self) -> Result<(), String> {
    if self.match_token(TokenType::Print) {
      self.print_statement()?;
    } else {
      self.expression_statement()?;
    }
    Ok(())
  }

  pub fn var_declaration(&mut self) -> Result<(), String> {
    let global = self.parse_variable("Expect variable name.")?;
    if self.match_token(TokenType::Equal) {
      self.expression()?;
    } else {
      self.chunk.emit_op(Op::Nil);
    }
    self.eat(
      TokenType::Semicolon,
      "Expect ';' after variable declaration.",
    )?;
    self.chunk.emit_define_global(global);
    Ok(())
  }

  pub fn declaration(&mut self) -> Result<(), String> {
    if self.match_token(TokenType::Var) {
      self.var_declaration()
    } else {
      self.statement()
    }
  }

  pub fn program(&mut self) -> Result<(), String> {
    while matches!(self.peek, Some(_)) {
      self.declaration()?;
    }
    Ok(())
  }

  pub fn parse_precedence(
    &mut self,
    precedence: Precedence,
  ) -> Result<(), String> {
    if let Some(token) = self.advance()? {
      let prefix =
        token.token_type.rule().prefix.ok_or("Expect expression.")?;
      let can_assign = precedence <= Precedence::Assignment;
      prefix(self, token, can_assign)?;

      while matches!(&self.peek, Some(p) if precedence <= p.token_type.rule().precedence)
      {
        if let Some(token) = self.advance()? {
          let infix =
            token.token_type.rule().infix.ok_or("Expect expression.")?;
          infix(self, token, can_assign)?;
        }
      }
      if can_assign && self.match_token(TokenType::Equal) {
        return Err("Invalid assignment target.".to_owned());
      }
    }
    Ok(())
  }

  pub fn parse_variable(&mut self, message: &str) -> Result<usize, String> {
    let token = self.eat(TokenType::Identifier, message)?;
    let name = &token.source;
    let id = self.chunk.add_constant(Value::string(name));
    Ok(id)
  }

  pub fn variable(
    &mut self,
    token: Token,
    can_assign: bool,
  ) -> Result<(), String> {
    let name = &token.source;
    let id = self.chunk.add_constant(Value::string(name));
    if can_assign && self.match_token(TokenType::Equal) {
      self.expression()?;
      self.chunk.emit_set_global(id);
    } else {
      self.chunk.emit_get_global(id);
    }
    Ok(())
  }

  pub fn grouping(
    &mut self,
    _token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    self.expression()?;
    self.eat(TokenType::RightParen, "Expect ')' after expression.")?;
    Ok(())
  }

  pub fn number(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    let constant = token
      .source
      .parse::<f64>()
      .map_err(|_e| "ParseFloatError".to_owned())?;
    self.chunk.emit_constant(Value::number(constant));
    Ok(())
  }

  pub fn string(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    let string = &token.source[1..(token.length - 1)];
    self.chunk.emit_constant(Value::string(string));
    Ok(())
  }

  pub fn literal(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    match token.token_type {
      TokenType::Nil => self.chunk.emit_op(Op::Nil),
      TokenType::False => self.chunk.emit_op(Op::False),
      TokenType::True => self.chunk.emit_op(Op::True),
      _ => unreachable!(),
    }
    Ok(())
  }

  pub fn unary(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    self.parse_precedence(Precedence::Unary)?;

    match token.token_type {
      TokenType::Bang => self.chunk.emit_op(Op::Not),
      TokenType::Minus => self.chunk.emit_op(Op::Negate),
      _ => unreachable!(),
    }
    Ok(())
  }

  pub fn binary(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
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
