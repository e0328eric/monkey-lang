use crate::error;

type Error = error::MonkeyErr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    ILLIGAL,
    EOF,
    // Identifiers + Literals
    IDENT(String),
    INT(i64),

    // Operations
    ASSIGN,
    PLUS,
    MINUS,
    BANG,
    ASTERISK,
    SLASH,
    POWER,
    LT,
    GT,
    EQ,
    NOTEQ,

    // Delimiters
    COMMA,
    SEMICOLON,

    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,

    // Keywords
    FUNCTION,
    LET,
    IF,
    ELSE,
    RETURN,
    TRUE,
    FALSE,
}

impl Token {
    pub fn is_str_keywords(s: &str) -> Option<Token> {
        match s {
            "let" => Some(Token::LET),
            "fn" => Some(Token::FUNCTION),
            "if" => Some(Token::IF),
            "else" => Some(Token::ELSE),
            "return" => Some(Token::RETURN),
            "true" => Some(Token::TRUE),
            "false" => Some(Token::FALSE),
            _ => None,
        }
    }

    pub fn unwrap_string(&self) -> error::Result<String> {
        if let Token::IDENT(ref s) = self {
            Ok(s.to_owned())
        } else {
            Err(Error::CannotConvertStringErr { got: self.clone() })
        }
    }

    pub fn take_tok_name(&self) -> String {
        match self {
            Token::IDENT(_) => "IDENT".to_string(),
            Token::INT(_) => "INT".to_string(),
            _ => format!("{:?}", self),
        }
    }

    pub fn to_symbol(&self) -> crate::error::Result<String> {
        match self {
            Token::PLUS => Ok("+".to_string()),
            Token::MINUS => Ok("-".to_string()),
            Token::ASTERISK => Ok("*".to_string()),
            Token::SLASH => Ok("/".to_string()),
            Token::LT => Ok("<".to_string()),
            Token::GT => Ok(">".to_string()),
            Token::EQ => Ok("==".to_string()),
            Token::NOTEQ => Ok("!=".to_string()),
            Token::POWER => Ok("**".to_string()),
            Token::BANG => Ok("!".to_string()),
            _ => Err(Error::CannotConvertSymbolErr { got: self.clone() }),
        }
    }
}

pub fn is_letter(chr: char) -> bool {
    chr.is_ascii_alphabetic() || chr == '_'
}

#[test]
fn check_is_letter() {
    assert!(is_letter('a'));
    assert!(is_letter('P'));
    assert!(is_letter('_'));
    assert!(!is_letter('3'));
    assert!(!is_letter('{'));
}
