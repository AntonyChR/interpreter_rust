#![allow(dead_code)]

use crate::token::{self, Token, TokenType};
use crate::utils::*;

pub const DOUBLE_QUOTE: &str = r#"""#;

#[derive(Copy, Clone)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    ch: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lexer: Lexer<'_> = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: "",
        };
        lexer.read_char(); //read first char
        lexer
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = "";
        } else {
            self.ch = &self.input[self.read_position..self.read_position + 1];
        }
        self.position = self.read_position;
        self.read_position = self.read_position + 1;
    }

    pub fn read_identifier(&mut self) -> &'a str {
        let position: usize = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }
        &self.input[position..self.position]
    }

    pub fn skip_white_space(&mut self) {
        while let Some(ch) = self.ch.chars().next() {
            if !matches!(ch, ' ' | '\t' | '\n' | '\r') {
                break;
            }
            self.read_char();
        }
    }

    pub fn read_number(&mut self) -> &'a str {
        let position: usize = self.position;
        while is_digit(&self.ch) {
            self.read_char();
        }
        &self.input[position..self.position]
    }

    pub fn read_string(&mut self) -> &'a str {
        let position: usize = self.position;
        loop{
            self.read_char();
            if self.ch == DOUBLE_QUOTE || self.read_position >= self.input.len(){
                break ;
            }
        }
        // +1 to skip first '"', We do not include the final '"'
        &self.input[position+1..self.position]
    }

    pub fn peek_char(&self) -> &'a str {
        if self.read_position >= self.input.len() {
            return "";
        } else {
            return &self.input[self.read_position..self.read_position + 1];
        }
    }

    pub fn next_token(&mut self) -> Token<'a> {
        let token: Token<'a>;
        self.skip_white_space();

        match self.ch {
            "=" => {
                if self.peek_char() == "=" {
                    self.read_char();
                    token = Token::new(TokenType::Eq, "==");
                } else {
                    token = Token::new(TokenType::Assign, self.ch);
                }
            }
            "!" => {
                if self.peek_char() == "=" {
                    self.read_char();
                    token = Token::new(TokenType::NotEq, "!=");
                } else {
                    token = Token::new(TokenType::Bang, self.ch)
                }
            }
            "+" => token = Token::new(TokenType::Plus, self.ch),
            "-" => token = Token::new(TokenType::Minus, self.ch),
            "/" => token = Token::new(TokenType::Slash, self.ch),
            "*" => token = Token::new(TokenType::Asterisk, self.ch),
            "<" => token = Token::new(TokenType::Lt, self.ch),
            ">" => token = Token::new(TokenType::Gt, self.ch),
            ";" => token = Token::new(TokenType::Semicolon, self.ch),
            "," => token = Token::new(TokenType::Comma, self.ch),
            "(" => token = Token::new(TokenType::Lparen, self.ch),
            ")" => token = Token::new(TokenType::Rparen, self.ch),
            "{" => token = Token::new(TokenType::Lbrace, self.ch),
            "}" => token = Token::new(TokenType::Rbrace, self.ch),
            DOUBLE_QUOTE => token = Token::new(TokenType::String, self.read_string()),
            "" => token = Token::new(TokenType::Eof, ""),
            _ => {
                if is_letter(self.ch) {
                    let literal = self.read_identifier();
                    return Token::new(token::lookup_identifier(literal), literal);
                } else if is_digit(self.ch) {
                    return Token::new(TokenType::Int, self.read_number());
                } else {
                    return Token::new(TokenType::Illegal, self.ch);
                }
            }
        }
        self.read_char();
        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn test_next_token() {
        let input = r#"
        let five = 5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };

        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;

        if ( 5 < 10 ){
            return true;
        }else{
            return false;
        }

        10 == 10;
        10 != 9;
        "foobar"
        "foo bar"
        "#;

        let expected = [
            //(expected type, expected literal)
            (TokenType::Let, "let"),
            (TokenType::Ident, "five"),
            (TokenType::Assign, "="),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "ten"),
            (TokenType::Assign, "="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "add"),
            (TokenType::Assign, "="),
            (TokenType::Function, "fn"),
            (TokenType::Lparen, "("),
            (TokenType::Ident, "x"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "y"),
            (TokenType::Rparen, ")"),
            (TokenType::Lbrace, "{"),
            (TokenType::Ident, "x"),
            (TokenType::Plus, "+"),
            (TokenType::Ident, "y"),
            (TokenType::Semicolon, ";"),
            (TokenType::Rbrace, "}"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "result"),
            (TokenType::Assign, "="),
            (TokenType::Ident, "add"),
            (TokenType::Lparen, "("),
            (TokenType::Ident, "five"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "ten"),
            (TokenType::Rparen, ")"),
            (TokenType::Semicolon, ";"),
            (TokenType::Bang, "!"),
            (TokenType::Minus, "-"),
            (TokenType::Slash, "/"),
            (TokenType::Asterisk, "*"),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::Gt, ">"),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::If, "if"),
            (TokenType::Lparen, "("),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::Rparen, ")"),
            (TokenType::Lbrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::True, "true"),
            (TokenType::Semicolon, ";"),
            (TokenType::Rbrace, "}"),
            (TokenType::Else, "else"),
            (TokenType::Lbrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::False, "false"),
            (TokenType::Semicolon, ";"),
            (TokenType::Rbrace, "}"),
            (TokenType::Int, "10"),
            (TokenType::Eq, "=="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),
            (TokenType::Int, "10"),
            (TokenType::NotEq, "!="),
            (TokenType::Int, "9"),
            (TokenType::Semicolon, ";"),
            (TokenType::String, "foobar"),
            (TokenType::String, "foo bar"),
            (TokenType::Eof, ""),
        ];

        let mut lexer = Lexer::new(input);
        for i in 0..expected.len() {
            let token = lexer.next_token();
            assert_eq!(
                token.toke_type, expected[i].0,
                "incorrect token type: at index {}",
                i
            );
            assert_eq!(
                token.literal, expected[i].1,
                "incorrect token literal: at index {}",
                i
            );
        }
    }
}
