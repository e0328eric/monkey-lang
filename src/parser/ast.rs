use crate::lexer::token::Token;

pub type Program = Vec<Statement>;
pub type BlockStmt = Vec<Statement>;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Statement {
    LetStmt { name: String, value: Expression },
    ReturnStmt { value: Expression },
    ExpressionStmt { expression: Expression },
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Expression {
    Ident(String),
    Boolean(bool),
    Integer(i64),
    // Yet a complex number is just implemented with integer values
    Complex {
        re: i64,
        im: i64,
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
        alternative: BlockStmt,
    },
    Function {
        parameter: Vec<String>,
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
            _ => Precedence::LOWEST,
        }
    }
}
