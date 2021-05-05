use super::*;

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
    "foobar";
    "foo bar";
    [1,2];
    { "foo": "bar" };
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
        Token::STRING("foobar".to_string()),
        Token::SEMICOLON,
        Token::STRING("foo bar".to_string()),
        Token::SEMICOLON,
        Token::LBRACKET,
        Token::INT(1),
        Token::COMMA,
        Token::INT(2),
        Token::RBRACKET,
        Token::SEMICOLON,
        Token::LBRACE,
        Token::STRING("foo".to_string()),
        Token::COLON,
        Token::STRING("bar".to_string()),
        Token::RBRACE,
        Token::SEMICOLON,
        Token::EOF,
    ];
    assert_eq!(lex, expected);
}
