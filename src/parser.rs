use crate::{
  chunk::Op,
  compiler::Compiler,
  inspector::Inspector,
  scanner::Scanner,
  scope::Upvalue,
  token::{Precedence, Token, TokenType},
  value::{Closure, Function, FunctionKind, Value},
  Chunk,
};

pub fn compile(source: &str) -> Result<Closure, String> {
  let scanner = Scanner::new(source);
  let mut parser = Parser::new(scanner, None);
  parser.advance()?; // TODO
  parser.program()?;
  let (closure, _) = parser.end_compiler();
  Ok(closure)
}

pub struct Parser<'source> {
  peek: Option<Token>,
  scanner: Scanner<'source>,
  compiler: Option<Compiler>,
  inspector: Option<Inspector>,
}

pub type ParseFn<'s> = fn(&mut Parser<'s>, Token, bool) -> Result<(), String>;

impl<'source> Parser<'source> {
  pub fn new(scanner: Scanner<'source>, inspector: Option<Inspector>) -> Self {
    Self {
      peek: None,
      scanner,
      compiler: Some(Compiler::script()),
      inspector,
    }
  }

  pub fn function_compiler(&mut self, name: &str) {
    self.compiler = Some(self.compiler.take().unwrap().function(name));
  }

  pub fn end_compiler(&mut self) -> (Closure, Vec<Upvalue>) {
    self.emitter().emit_op(Op::Nil);
    self.emitter().emit_op(Op::Return);
    let (enclosing, function, upvalues) = self.compiler.take().unwrap().end();
    self.compiler = enclosing;
    if let Some(ref mut inspector) = self.inspector {
      inspector.catch_bytecode(function.clone());
    }
    (Closure::new(function, upvalues.len() as u8), upvalues)
  }

  pub fn into_inspector(self) -> Option<Inspector> {
    self.inspector
  }

  fn get_compiler_mut(&mut self) -> &mut Compiler {
    self
      .compiler
      .as_mut()
      .expect("use compiler before end_compiler")
  }

  fn emitter(&mut self) -> &mut Chunk {
    self.get_compiler_mut().chunk()
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
    self.emitter().emit_op(Op::Print);
    Ok(())
  }

  fn expression_statement(&mut self) -> Result<(), String> {
    self.expression()?;
    self.eat(TokenType::Semicolon, "Expect ';' after expression.")?;
    self.emitter().emit_op(Op::Pop);
    Ok(())
  }

  fn if_statement(&mut self) -> Result<(), String> {
    self.eat(TokenType::LeftParen, "Expect '(' after 'if'.")?;
    self.expression()?;
    self.eat(TokenType::RightParen, "Expect ')' after condition.")?;

    let then_jump = self.emitter().emit_jump(Op::JumpIfFalse)?;
    self.emitter().emit_op(Op::Pop);
    self.statement()?;

    let else_jump = self.emitter().emit_jump(Op::Jump)?;

    self.emitter().patch_jump(then_jump)?;
    self.emitter().emit_op(Op::Pop);

    if self.match_token(TokenType::Else) {
      self.statement()?;
    }
    self.emitter().patch_jump(else_jump)?;

    Ok(())
  }

  fn return_statement(&mut self) -> Result<(), String> {
    if let FunctionKind::Script = self.get_compiler_mut().function.kind {
      return Err("Can't return from top-level code.".to_owned());
    }

    if self.match_token(TokenType::Semicolon) {
      self.emitter().emit_op(Op::Nil);
      self.emitter().emit_op(Op::Return);
    } else {
      self.expression()?;
      self.eat(TokenType::Semicolon, "Expect ';' after return value.")?;
      self.emitter().emit_op(Op::Return);
    }
    Ok(())
  }

  fn while_statement(&mut self) -> Result<(), String> {
    let loop_start = self.emitter().code_len()?;
    self.eat(TokenType::LeftParen, "Expect '(' after 'while'.")?;
    self.expression()?;
    self.eat(TokenType::RightParen, "Expect ')' after condition.")?;

    let exit_jump = self.emitter().emit_jump(Op::JumpIfFalse)?;
    self.emitter().emit_op(Op::Pop);
    self.statement()?;
    self.emitter().emit_loop(loop_start)?;

    self.emitter().patch_jump(exit_jump)?;
    self.emitter().emit_op(Op::Pop);

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

    let mut loop_start = self.emitter().code_len()?;

    let mut exit_jump = None;
    if !self.match_token(TokenType::Semicolon) {
      self.expression()?;
      self.eat(TokenType::Semicolon, "Expect ';' after loop condition.")?;

      exit_jump = Some(self.emitter().emit_jump(Op::JumpIfFalse)?);
      self.emitter().emit_op(Op::Pop);
    }

    if !self.match_token(TokenType::RightParen) {
      let body_jump = self.emitter().emit_jump(Op::Jump)?;
      let increment_start = self.emitter().code_len()?;
      self.expression()?;
      self.emitter().emit_op(Op::Pop);
      self.eat(TokenType::RightParen, "Expect ')' after for clauses.")?;

      self.emitter().emit_loop(loop_start)?;
      loop_start = increment_start;
      self.emitter().patch_jump(body_jump)?;
    }

    self.statement()?;
    self.emitter().emit_loop(loop_start)?;

    if let Some(exit_jump) = exit_jump {
      self.emitter().patch_jump(exit_jump)?;
      self.emitter().emit_op(Op::Pop);
    }

    self.end_scope();
    Ok(())
  }

  fn begin_scope(&mut self) {
    self.get_compiler_mut().scopes.push();
  }

  fn end_scope(&mut self) {
    let compiler = self.get_compiler_mut();
    let scope = compiler.scopes.pop().unwrap();

    for _ in 0..scope.len() {
      compiler.chunk().emit_op(Op::Pop);
    }
  }

  fn statement(&mut self) -> Result<(), String> {
    if self.match_token(TokenType::Print) {
      self.print_statement()?;
    } else if self.match_token(TokenType::If) {
      self.if_statement()?;
    } else if self.match_token(TokenType::Return) {
      self.return_statement()?;
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

  fn function(&mut self, name: &str) -> Result<(), String> {
    self.function_compiler(name);
    self.begin_scope();

    self.eat(TokenType::LeftParen, "Expect '(' after function name.")?;
    if !self.check(TokenType::RightParen) {
      loop {
        self.get_compiler_mut().function.arity = self
          .get_compiler_mut()
          .function
          .arity
          .checked_add(1)
          .ok_or("Can't have more than 255 parameters.")?;
        let token =
          self.eat(TokenType::Identifier, "Expect parameter name.")?;
        let name = &token.source;
        self.parse_local_variable(name)?;
        self.get_compiler_mut().scopes.mark_init_local(name);

        if !self.match_token(TokenType::Comma) {
          break;
        }
      }
    }
    self.eat(TokenType::RightParen, "Expect ')' after parameters.")?;
    self.eat(TokenType::LeftBrace, "Expect '{' before function body.")?;

    self.block()?;

    let (function, upvalues) = self.end_compiler();
    self.emitter().emit_closure(function)?;

    self.get_compiler_mut().emit_upvalues(upvalues);
    Ok(())
  }

  fn fun_declaration(&mut self) -> Result<(), String> {
    let token = self.eat(TokenType::Identifier, "Expect function name.")?;
    let name = &token.source;

    let global = if self.get_compiler_mut().scopes.is_empty() {
      let global = self.emitter().add_constant(Value::string(name))?;
      Some(global)
    } else {
      self.parse_local_variable(name)?;
      self.get_compiler_mut().scopes.mark_init_local(name);
      None
    };

    self.function(name)?;

    if let Some(global) = global {
      self.emitter().emit_define_global(global);
    }

    Ok(())
  }

  fn parse_local_variable(&mut self, name: &str) -> Result<(), String> {
    if self.get_compiler_mut().scopes.current_has(name).unwrap() {
      Err("Already a variable with this name in this scope.".to_owned())
    } else {
      self
        .get_compiler_mut()
        .scopes
        .define_uninit_local(name.to_owned())?;
      Ok(())
    }
  }

  fn var_declaration(&mut self) -> Result<(), String> {
    let token = self.eat(TokenType::Identifier, "Expect variable name.")?;
    let name = &token.source;

    let global = if self.get_compiler_mut().scopes.is_empty() {
      let global = self.emitter().add_constant(Value::string(name))?;
      Some(global)
    } else {
      self.parse_local_variable(name)?;
      None
    };

    if self.match_token(TokenType::Equal) {
      self.expression()?;
    } else {
      self.emitter().emit_op(Op::Nil);
    }
    self.eat(
      TokenType::Semicolon,
      "Expect ';' after variable declaration.",
    )?;

    match global {
      Some(global) => self.emitter().emit_define_global(global),
      None => self.get_compiler_mut().scopes.mark_init_local(name),
    }
    Ok(())
  }

  fn declaration(&mut self) -> Result<(), String> {
    if self.match_token(TokenType::Fun) {
      self.fun_declaration()
    } else if self.match_token(TokenType::Var) {
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
    let local = self.get_compiler_mut().scopes.resolve_local(name)?;
    match (is_set, local) {
      (true, None) => {
        if let Some(upvalue) = self.get_compiler_mut().resolve_upvalue(name)? {
          self.expression()?;
          self.emitter().emit_set_upvalue(upvalue);
        } else {
          let global = self.emitter().add_constant(Value::string(name))?;
          self.expression()?;
          self.emitter().emit_set_global(global);
        }
      }
      (false, None) => {
        if let Some(upvalue) = self.get_compiler_mut().resolve_upvalue(name)? {
          self.emitter().emit_get_upvalue(upvalue);
        } else {
          let global = self.emitter().add_constant(Value::string(name))?;
          self.emitter().emit_get_global(global);
        }
      }
      (true, Some(local)) => {
        let index = local.index;
        self.expression()?;
        self.emitter().emit_set_local(index);
      }
      (false, Some(local)) => {
        let index = local.index;
        self.emitter().emit_get_local(index);
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
    self.emitter().emit_constant(Value::number(constant))?;
    Ok(())
  }

  pub fn string(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    let string = &token.source[1..(token.length - 1)];
    self.emitter().emit_constant(Value::string(string))?;
    Ok(())
  }

  pub fn literal(
    &mut self,
    token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    match token.token_type {
      TokenType::Nil => self.emitter().emit_op(Op::Nil),
      TokenType::False => self.emitter().emit_op(Op::False),
      TokenType::True => self.emitter().emit_op(Op::True),
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
      TokenType::Bang => self.emitter().emit_op(Op::Not),
      TokenType::Minus => self.emitter().emit_op(Op::Negate),
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
        self.emitter().emit_op(Op::Equal);
        self.emitter().emit_op(Op::Not);
      }
      TokenType::EqualEqual => self.emitter().emit_op(Op::Equal),
      TokenType::Greater => self.emitter().emit_op(Op::Greater),
      TokenType::GreaterEqual => {
        self.emitter().emit_op(Op::Less);
        self.emitter().emit_op(Op::Not);
      }
      TokenType::Less => self.emitter().emit_op(Op::Less),
      TokenType::LessEqual => {
        self.emitter().emit_op(Op::Greater);
        self.emitter().emit_op(Op::Not);
      }
      TokenType::Plus => self.emitter().emit_op(Op::Add),
      TokenType::Minus => self.emitter().emit_op(Op::Subtract),
      TokenType::Star => self.emitter().emit_op(Op::Multiply),
      TokenType::Slash => self.emitter().emit_op(Op::Divide),
      _ => unreachable!(),
    }
    Ok(())
  }

  pub fn and(
    &mut self,
    _token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    let end_jump = self.emitter().emit_jump(Op::JumpIfFalse)?;
    self.emitter().emit_op(Op::Pop);
    self.parse_precedence(Precedence::And)?;
    self.emitter().patch_jump(end_jump)?;
    Ok(())
  }

  pub fn or(&mut self, _token: Token, _can_assign: bool) -> Result<(), String> {
    let else_jump = self.emitter().emit_jump(Op::JumpIfFalse)?;
    let end_jump = self.emitter().emit_jump(Op::Jump)?;
    self.emitter().patch_jump(else_jump)?;
    self.emitter().emit_op(Op::Pop);
    self.parse_precedence(Precedence::Or)?;
    self.emitter().patch_jump(end_jump)?;
    Ok(())
  }

  pub fn call(
    &mut self,
    _token: Token,
    _can_assign: bool,
  ) -> Result<(), String> {
    let mut arg_count: u8 = 0;
    if !self.check(TokenType::RightParen) {
      loop {
        self.expression()?;
        arg_count = arg_count
          .checked_add(1)
          .ok_or("Can't have more than 255 arguments.")?;

        if !self.match_token(TokenType::Comma) {
          break;
        }
      }
    }
    self.eat(TokenType::RightParen, "Expect ')' after arguments.")?;
    self.emitter().emit_call(arg_count);
    Ok(())
  }
}
