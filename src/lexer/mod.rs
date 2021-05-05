#[cfg(test)]
mod lexer_test;
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
            ':' => Token::COLON,
            '(' => Token::LPAREN,
            ')' => Token::RPAREN,
            '{' => Token::LBRACE,
            '}' => Token::RBRACE,
            '[' => Token::LBRACKET,
            ']' => Token::RBRACKET,
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
