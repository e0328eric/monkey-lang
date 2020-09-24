pub mod token;
use crate::lexer::token::Token;

#[derive(Debug)]
pub struct Lexer<'a> {
    pub input: &'a str,
    pub position: usize,
    pub read_position: usize,
    pub ch: char,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lex = Self {
            input,
            position: 0,
            read_position: 0,
            ch: '\x00',
        };
        lex.read_char();
        lex
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\x00'
        } else {
            self.ch = self.input.as_bytes()[self.read_position] as char;
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&mut self) -> char {
        if self.read_position >= self.input.len() {
            '\x00'
        } else {
            self.input.as_bytes()[self.read_position] as char
        }
    }

    pub fn next_token(&mut self) -> Token {
        while self.ch.is_whitespace() {
            self.read_char()
        }
        let tok = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::EQ
                } else {
                    Token::ASSIGN
                }
            }
            '+' => Token::PLUS,
            '-' => Token::MINUS,
            '*' => {
                if self.peek_char() == '*' {
                    self.read_char();
                    Token::POWER
                } else {
                    Token::ASTERISK
                }
            }
            '/' => Token::SLASH,
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NOTEQ
                } else {
                    Token::BANG
                }
            }
            '<' => Token::LT,
            '>' => Token::GT,
            ',' => Token::COMMA,
            ';' => Token::SEMICOLON,
            '(' => Token::LPAREN,
            ')' => Token::RPAREN,
            '{' => Token::LBRACE,
            '}' => Token::RBRACE,
            '"' => self.read_string(),
            '\x00' => Token::EOF,
            _ if token::is_letter(self.ch) => {
                let read_str = self.read_identifier();
                let semi_tok = Token::is_str_keywords(&read_str);
                if let Some(tmp) = semi_tok {
                    tmp
                } else {
                    Token::IDENT(read_str.to_string())
                }
            }
            _ if self.ch.is_ascii_digit() => self.read_number(),
            _ => Token::ILLIGAL,
        };
        self.read_char();
        tok
    }

    fn read_identifier(&mut self) -> &str {
        let position = self.position;
        while token::is_letter(self.ch) {
            self.read_char();
        }
        self.position -= 1;
        self.read_position -= 1;
        &self.input[position..=self.position]
    }

    fn read_string(&mut self) -> Token {
        let position = self.position + 1;
        loop {
            self.read_char();
            if self.ch == '"' || self.ch == '\x00' {
                break;
            }
        }
        Token::STRING(self.input[position..self.position].to_string())
    }

    // Add lexing an imeginary part of complex number
    fn read_number(&mut self) -> Token {
        let position = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        self.position -= 1;
        self.read_position -= 1;
        let num = self.input[position..=self.position].parse().unwrap();
        if self.peek_char() == 'i' {
            self.read_char();
            Token::IMEGINARY(num)
        } else {
            Token::INT(num)
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.position > self.input.len() {
            None
        } else {
            Some(self.next_token())
        }
    }
}

#[test]
fn test_next_token() {
    let input = "=+(){},;";
    let lex = Lexer::new(&input).collect::<Vec<Token>>();
    let expected = vec![
        Token::ASSIGN,
        Token::PLUS,
        Token::LPAREN,
        Token::RPAREN,
        Token::LBRACE,
        Token::RBRACE,
        Token::COMMA,
        Token::SEMICOLON,
        Token::EOF,
    ];
    assert_eq!(lex, expected);
}

#[test]
fn more_complex_lex() {
    let input = r#"let five = 5;
    let ten = 10;
    let add = fn(x, y) {
        x + y;
    };
    let result = add(five, ten);
    !-/*5;
    5 < 10 > 5;
    10 ** 10;
    
    if (5 < 10) {
        return true;
    } else {
        return false;
    }
    
    10 == 10;
    10 != 9;
    1 + 2i;
    "#;
    let lex = Lexer::new(&input).collect::<Vec<Token>>();
    let expected = vec![
        Token::LET,
        Token::IDENT("five".to_string()),
        Token::ASSIGN,
        Token::INT(5),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("ten".to_string()),
        Token::ASSIGN,
        Token::INT(10),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("add".to_string()),
        Token::ASSIGN,
        Token::FUNCTION,
        Token::LPAREN,
        Token::IDENT("x".to_string()),
        Token::COMMA,
        Token::IDENT("y".to_string()),
        Token::RPAREN,
        Token::LBRACE,
        Token::IDENT("x".to_string()),
        Token::PLUS,
        Token::IDENT("y".to_string()),
        Token::SEMICOLON,
        Token::RBRACE,
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("result".to_string()),
        Token::ASSIGN,
        Token::IDENT("add".to_string()),
        Token::LPAREN,
        Token::IDENT("five".to_string()),
        Token::COMMA,
        Token::IDENT("ten".to_string()),
        Token::RPAREN,
        Token::SEMICOLON,
        Token::BANG,
        Token::MINUS,
        Token::SLASH,
        Token::ASTERISK,
        Token::INT(5),
        Token::SEMICOLON,
        Token::INT(5),
        Token::LT,
        Token::INT(10),
        Token::GT,
        Token::INT(5),
        Token::SEMICOLON,
        Token::INT(10),
        Token::POWER,
        Token::INT(10),
        Token::SEMICOLON,
        Token::IF,
        Token::LPAREN,
        Token::INT(5),
        Token::LT,
        Token::INT(10),
        Token::RPAREN,
        Token::LBRACE,
        Token::RETURN,
        Token::TRUE,
        Token::SEMICOLON,
        Token::RBRACE,
        Token::ELSE,
        Token::LBRACE,
        Token::RETURN,
        Token::FALSE,
        Token::SEMICOLON,
        Token::RBRACE,
        Token::INT(10),
        Token::EQ,
        Token::INT(10),
        Token::SEMICOLON,
        Token::INT(10),
        Token::NOTEQ,
        Token::INT(9),
        Token::SEMICOLON,
        Token::INT(1),
        Token::PLUS,
        Token::IMEGINARY(2),
        Token::SEMICOLON,
        Token::EOF,
    ];
    assert_eq!(lex, expected);
}
