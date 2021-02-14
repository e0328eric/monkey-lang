use crate::lexer::token::Token;

pub type BlockStmt = Vec<Statement>;

#[repr(transparent)]
#[derive(Debug)]
pub struct Program {
  stmts: Vec<Statement>,
}

impl Program {
  pub fn new(stmts: Vec<Statement>) -> Self {
    Self { stmts }
  }

  pub fn get_stmts(&self) -> &Vec<Statement> {
    &self.stmts
  }

  pub fn push(&mut self, stmt: Statement) {
    self.stmts.push(stmt);
  }
}

impl PartialEq<Program> for Vec<Statement> {
  fn eq(&self, other: &Program) -> bool {
    self == &other.stmts
  }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Statement {
  LetStmt { name: String, value: Expression },
  ReturnStmt { value: Expression },
  ExpressionStmt { expression: Expression },
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Expression {
  Ident(String),
  String(String),
  Boolean(bool),
  Integer(i64),
  Array(Vec<Expression>),
  // Yet a complex number is just implemented with integer values
  Complex {
    re: i64,
    im: i64,
  },
  Hash {
    key: Vec<Expression>,
    value: Vec<Expression>,
  },
  Prefix {
    operator: Token,
    right: Box<Expression>,
  },
  Infix {
    left: Box<Expression>,
    operator: Token,
    right: Box<Expression>,
  },
  IfExpr {
    condition: Box<Expression>,
    consequence: BlockStmt,
    alternative: Option<BlockStmt>,
  },
  Function {
    parameters: Vec<String>,
    body: BlockStmt,
  },
  Call {
    function: Box<Expression>,
    arguments: Vec<Expression>,
  },
}

impl From<Box<Expression>> for Statement {
  fn from(expr: Box<Expression>) -> Self {
    Self::ExpressionStmt { expression: *expr }
  }
}

impl From<Expression> for Statement {
  fn from(expr: Expression) -> Self {
    Self::ExpressionStmt { expression: expr }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Precedence {
  LOWEST,
  EQUALS,
  LESSGREATER,
  SUM,
  PRODUCT,
  POWER,
  PREFIX,
  CALL,
  INDEX,
}

impl Precedence {
  pub fn take_precedence(tok: &Token) -> Self {
    match tok {
      Token::EQ => Precedence::EQUALS,
      Token::NOTEQ => Precedence::EQUALS,
      Token::LT => Precedence::LESSGREATER,
      Token::GT => Precedence::LESSGREATER,
      Token::PLUS => Precedence::SUM,
      Token::MINUS => Precedence::SUM,
      Token::ASTERISK => Precedence::PRODUCT,
      Token::SLASH => Precedence::PRODUCT,
      Token::POWER => Precedence::POWER,
      Token::LPAREN => Precedence::CALL,
      Token::LBRACKET => Precedence::INDEX,
      _ => Precedence::LOWEST,
    }
  }
}
