use crate::error;

type Error = error::MonkeyErr;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    ILLIGAL,
    EOF,
    // Identifiers + Literals
    IDENT(String),
    STRING(String),
    INT(i64),
    IMEGINARY(i64),

    // Operations
    ASSIGN,   // =
    PLUS,     // +
    MINUS,    // -
    BANG,     // !
    ASTERISK, // *
    SLASH,    // /
    POWER,    // **
    LT,       // <
    GT,       // >
    EQ,       // ==
    NOTEQ,    // !=

    // Delimiters
    COMMA,     // ,
    SEMICOLON, // ;
    COLON,     // :

    LPAREN,   // (
    RPAREN,   // )
    LBRACE,   // {
    RBRACE,   // }
    LBRACKET, // [
    RBRACKET, // ]

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

    pub fn is_same_type(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::IDENT(_), Token::IDENT(_)) => true,
            (Token::STRING(_), Token::STRING(_)) => true,
            (Token::INT(_), Token::INT(_)) => true,
            (Token::IMEGINARY(_), Token::IMEGINARY(_)) => true,
            _ => self == other,
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
            Token::STRING(_) => "STRING".to_string(),
            Token::INT(_) => "INT".to_string(),
            Token::IMEGINARY(_) => "IMEGINARY".to_string(),
            _ => format!("{:?}", self),
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
