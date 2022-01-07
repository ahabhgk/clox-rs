use crate::{
  chunk::{Chunk, Op},
  compiler::Compiler,
  scanner::Scanner,
  token::{Precedence, Token, TokenType},
  value::Value,
};

pub fn compile(source: &str) -> Result<Chunk, String> {
  let scanner = Scanner::new(source);
  let mut parser = Parser::new(scanner);
  parser.advance()?; // TODO
  parser.program()?;
  let chunk = parser.end();
  Ok(chunk)
}

pub struct Parser<'source> {
  peek: Option<Token>,
  scanner: Scanner<'source>,
  compiler: Compiler,
}

pub type ParseFn<'s> = fn(&mut Parser<'s>, Token, bool) -> Result<(), String>;

impl<'source> Parser<'source> {
  pub fn new(scanner: Scanner<'source>) -> Self {
    Self {
      peek: None,
      scanner,
      compiler: Compiler::new(),
    }
  }

  pub fn end(mut self) -> Chunk {
    self.compiler.chunk.emit_op(Op::Return);
    self.compiler.chunk
  }

  pub fn advance(&mut self) -> Result<Option<Token>, String> {
    let current = self.peek.take();
    self.peek = self.scanner.scan_token()?;
    Ok(current)
  }

  fn eat(
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

  fn match_token(&mut self, token_type: TokenType) -> bool {
    self.eat(token_type, "").is_ok()
  }

  fn is_end(&self) -> bool {
    matches!(self.peek, None)
  }

  fn check(&self, token_type: TokenType) -> bool {
    matches!(&self.peek, Some(p) if p.token_type == token_type)
  }

  fn expression(&mut self) -> Result<(), String> {
    self.parse_precedence(Precedence::Assignment)
  }

  fn print_statement(&mut self) -> Result<(), String> {
    self.expression()?;
    self.eat(TokenType::Semicolon, "Expect ';' after value.")?;
    self.compiler.chunk.emit_op(Op::Print);
    Ok(())
  }

  fn expression_statement(&mut self) -> Result<(), String> {
    self.expression()?;
    self.eat(TokenType::Semicolon, "Expect ';' after expression.")?;
    self.compiler.chunk.emit_op(Op::Pop);
    Ok(())
  }

  fn if_statement(&mut self) -> Result<(), String> {
    self.eat(TokenType::LeftParen, "Expect '(' after 'if'.")?;
    self.expression()?;
    self.eat(TokenType::RightParen, "Expect ')' after condition.")?;

    let then_jump = self.compiler.chunk.emit_jump(Op::JumpIfFalse)?;
    self.compiler.chunk.emit_op(Op::Pop);
    self.statement()?;

    let else_jump = self.compiler.chunk.emit_jump(Op::Jump)?;

    self.compiler.chunk.patch_jump(then_jump)?;
    self.compiler.chunk.emit_op(Op::Pop);

    if self.match_token(TokenType::Else) {
      self.statement()?;
    }
    self.compiler.chunk.patch_jump(else_jump)?;

    Ok(())
  }

  fn while_statement(&mut self) -> Result<(), String> {
    let loop_start = self.compiler.chunk.code_len()?;
    self.eat(TokenType::LeftParen, "Expect '(' after 'while'.")?;
    self.expression()?;
    self.eat(TokenType::RightParen, "Expect ')' after condition.")?;

    let exit_jump = self.compiler.chunk.emit_jump(Op::JumpIfFalse)?;
    self.compiler.chunk.emit_op(Op::Pop);
    self.statement()?;
    self.compiler.chunk.emit_loop(loop_start)?;

    self.compiler.chunk.patch_jump(exit_jump)?;
    self.compiler.chunk.emit_op(Op::Pop);

    Ok(())
  }

  fn for_statement(&mut self) -> Result<(), String> {
    self.begin_scope();

    self.eat(TokenType::LeftParen, "Expect '(' after 'for'.")?;
    if self.match_token(TokenType::Semicolon) {
      // No initializer
    } else if self.match_token(TokenType::Var) {
      self.var_declaration()?;
    } else {
      self.expression_statement()?;
    }

    let mut loop_start = self.compiler.chunk.code_len()?;

    let mut exit_jump = None;
    if !self.match_token(TokenType::Semicolon) {
      self.expression()?;
      self.eat(TokenType::Semicolon, "Expect ';' after loop condition.")?;

      exit_jump = Some(self.compiler.chunk.emit_jump(Op::JumpIfFalse)?);
      self.compiler.chunk.emit_op(Op::Pop);
    }

    if !self.match_token(TokenType::RightParen) {
      let body_jump = self.compiler.chunk.emit_jump(Op::Jump)?;
      let increment_start = self.compiler.chunk.code_len()?;
      self.expression()?;
      self.compiler.chunk.emit_op(Op::Pop);
      self.eat(TokenType::RightParen, "Expect ')' after for clauses.")?;

      self.compiler.chunk.emit_loop(loop_start)?;
      loop_start = increment_start;
      self.compiler.chunk.patch_jump(body_jump)?;
    }

    self.statement()?;
    self.compiler.chunk.emit_loop(loop_start)?;

    if let Some(exit_jump) = exit_jump {
      self.compiler.chunk.patch_jump(exit_jump)?;
      self.compiler.chunk.emit_op(Op::Pop);
    }

    self.end_scope();
    Ok(())
  }

  fn begin_scope(&mut self) {
    self.compiler.scopes.push();
  }

  fn end_scope(&mut self) {
    let scope = self.compiler.scopes.pop().unwrap();

    for _ in 0..scope.len() {
      self.compiler.chunk.emit_op(Op::Pop);
    }
  }

  fn statement(&mut self) -> Result<(), String> {
    if self.match_token(TokenType::Print) {
      self.print_statement()?;
    } else if self.match_token(TokenType::If) {
      self.if_statement()?;
    } else if self.match_token(TokenType::While) {
      self.while_statement()?;
    } else if self.match_token(TokenType::For) {
      self.for_statement()?;
    } else if self.match_token(TokenType::LeftBrace) {
      self.begin_scope();
      self.block()?;
      self.end_scope();
    } else {
      self.expression_statement()?;
    }
    Ok(())
  }

  fn block(&mut self) -> Result<(), String> {
    while !self.is_end() && !self.check(TokenType::RightBrace) {
      self.declaration()?;
    }

    self.eat(TokenType::RightBrace, "Expect '}' after block.")?;
    Ok(())
  }

  fn var_declaration(&mut self) -> Result<(), String> {
    let token = self.eat(TokenType::Identifier, "Expect variable name.")?;
    let name = &token.source;

    let global = if self.compiler.scopes.is_empty() {
      let global = self.compiler.chunk.add_constant(Value::string(name))?;
      Some(global)
    } else {
      if self.compiler.scopes.current_has(name).unwrap() {
        return Err(
          "Already a variable with this name in this scope.".to_owned(),
        );
      } else {
        self.compiler.scopes.define_uninit_local(name.to_owned())?;
        None
      }
    };

    if self.match_token(TokenType::Equal) {
      self.expression()?;
    } else {
      self.compiler.chunk.emit_op(Op::Nil);
    }
    self.eat(
      TokenType::Semicolon,
      "Expect ';' after variable declaration.",
    )?;

    match global {
      Some(global) => self.compiler.chunk.emit_define_global(global),
      None => self.compiler.scopes.mark_init_local(name),
    }
    Ok(())
  }

  fn declaration(&mut self) -> Result<(), String> {
    if self.match_token(TokenType::Var) {
      self.var_declaration()
    } else {
      self.statement()
    }
  }

  pub fn program(&mut self) -> Result<(), String> {
    while !self.is_end() {
      self.declaration()?;
    }
    Ok(())
  }

  fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), String> {
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

  pub fn variable(
    &mut self,
    token: Token,
    can_assign: bool,
  ) -> Result<(), String> {
    let is_set = can_assign && self.match_token(TokenType::Equal);
    let name = &token.source;
    let local = self.compiler.scopes.resolve_local(name)?;
    match (is_set, local) {
      (true, None) => {
        let name = &token.source;
        let global = self.compiler.chunk.add_constant(Value::string(name))?;
        self.expression()?;
        self.compiler.chunk.emit_set_global(global);
      }
      (false, None) => {
        let name = &token.source;
        let global = self.compiler.chunk.add_constant(Value::string(name))?;
        self.compiler.chunk.emit_get_global(global);
      }
      (true, Some(local)) => {
        let index = local.index;
        self.expression()?;
        self.compiler.chunk.emit_set_local(index);
      }
      (false, Some(local)) => {
        self.compiler.chunk.emit_get_local(local.index);
      }
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
    self.compiler.chunk.emit_constant(Value::number(constant))?;
    Ok(())
  }

  pub fn string(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    let string = &token.source[1..(token.length - 1)];
    self.compiler.chunk.emit_constant(Value::string(string))?;
    Ok(())
  }

  pub fn literal(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    match token.token_type {
      TokenType::Nil => self.compiler.chunk.emit_op(Op::Nil),
      TokenType::False => self.compiler.chunk.emit_op(Op::False),
      TokenType::True => self.compiler.chunk.emit_op(Op::True),
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
      TokenType::Bang => self.compiler.chunk.emit_op(Op::Not),
      TokenType::Minus => self.compiler.chunk.emit_op(Op::Negate),
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
        self.compiler.chunk.emit_op(Op::Equal);
        self.compiler.chunk.emit_op(Op::Not);
      }
      TokenType::EqualEqual => self.compiler.chunk.emit_op(Op::Equal),
      TokenType::Greater => self.compiler.chunk.emit_op(Op::Greater),
      TokenType::GreaterEqual => {
        self.compiler.chunk.emit_op(Op::Less);
        self.compiler.chunk.emit_op(Op::Not);
      }
      TokenType::Less => self.compiler.chunk.emit_op(Op::Less),
      TokenType::LessEqual => {
        self.compiler.chunk.emit_op(Op::Greater);
        self.compiler.chunk.emit_op(Op::Not);
      }
      TokenType::Plus => self.compiler.chunk.emit_op(Op::Add),
      TokenType::Minus => self.compiler.chunk.emit_op(Op::Subtract),
      TokenType::Star => self.compiler.chunk.emit_op(Op::Multiply),
      TokenType::Slash => self.compiler.chunk.emit_op(Op::Divide),
      _ => unreachable!(),
    }
    Ok(())
  }

  pub fn and(
    &mut self,
    _token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    let end_jump = self.compiler.chunk.emit_jump(Op::JumpIfFalse)?;
    self.compiler.chunk.emit_op(Op::Pop);
    self.parse_precedence(Precedence::And)?;
    self.compiler.chunk.patch_jump(end_jump)?;
    Ok(())
  }

  pub fn or(&mut self, _token: Token, _can_assign: bool) -> Result<(), String> {
    let else_jump = self.compiler.chunk.emit_jump(Op::JumpIfFalse)?;
    let end_jump = self.compiler.chunk.emit_jump(Op::Jump)?;
    self.compiler.chunk.patch_jump(else_jump)?;
    self.compiler.chunk.emit_op(Op::Pop);
    self.parse_precedence(Precedence::Or)?;
    self.compiler.chunk.patch_jump(end_jump)?;
    Ok(())
  }
}
