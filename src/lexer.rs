#![allow(dead_code)]

use crate::token::*;
use crate::utils::*;

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    ch: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lexer = Lexer {
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

    pub fn read_identifier(&mut self) -> &str{
        let position :usize = self.position;
        while is_letter(self.ch) {
            self.read_char(); 
        }

        return &self.input[position..self.position];
    }

    pub fn skip_white_space(&mut self){
        while let Some(ch) = self.ch.chars().next() {
            if !matches!(ch, ' ' | '\t' | '\n' | '\r') {
                break;
            }
            self.read_char();
        }
   }

    pub fn read_number(&mut self)->&str{
        let position = self.position;
        while is_digit(&self.ch) {
            self.read_char();            
        }
        return &self.input[position..self.position];
    }

    pub fn peek_char(&self) -> &'a str{
        if self.read_position >= self.input.len(){
            return "";
        }else{
            return &self.input[self.read_position..self.read_position+1];
        } 
    }

    pub fn next_token(&mut self) -> Token{
        let tok: Token;
        self.skip_white_space();

        match self.ch {
            "=" =>{
                if self.peek_char() == "=" {
                    //let ch = self.ch;
                    self.read_char();
                    //let literal = format!("{}{}",ch, self.ch);
                    //let literal = [ch, self.ch].join("");
                    //tok = Token::new(EQ,literal); 
                    tok = Token::new(EQ, "==");
                }else{
                    tok = Token::new(ASSIGN, self.ch);
                }
            },
            "!" => {
                if self.peek_char() == "=" {
                    //let ch = self.ch;
                    self.read_char();
                    //let literal = format!("{}{}",ch, self.ch);
                    //let literal = [ch, self.ch].join("");
                    //tok = Token::new(NOT_EQ, literal);
                    tok = Token::new(NOT_EQ, "!=");
                }else{
                    tok = Token::new(BANG, self.ch)
                }
            },
           "+" => tok = Token::new(PLUS, self.ch),
           "-" => tok = Token::new(MINUS, self.ch),
           "/" => tok = Token::new(SLASH, self.ch),
           "*" => tok = Token::new(ASTERISK, self.ch),
           "<" => tok = Token::new(LT, self.ch),
           ">" => tok = Token::new(GT, self.ch),
           ";" => tok = Token::new(SEMICOLON, self.ch),
           "," => tok = Token::new(COMMA, self.ch),
           "(" => tok = Token::new(LPAREN, self.ch),
           ")" => tok = Token::new(RPAREN, self.ch),
           "{" => tok = Token::new(LBRACE, self.ch),
           "}" => tok = Token::new(RBRACE, self.ch),
           "" => tok = Token::new(EOF, ""),
           _ => {
               if is_letter(self.ch) {
                   let literal = self.read_identifier();
                   tok = Token::new(lookup_identifier(literal),literal );
                   return tok;
               }else if is_digit(self.ch){
                   tok = Token::new(INT, self.read_number());
                   return tok;
               }else{
                   return Token::new(ILLEGAL, &self.ch);
               }
           } 
        }

        self.read_char();
        return tok; 
    }

}

mod tests {
    use crate::token::*;
    use crate::lexer::*;
    #[test]
    fn test_next_token() {
        let input = "
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
        ";

        let tests = [
            //(expected type, expected literal)
            (LET, "let"),
            (IDENT, "five"),
            (ASSIGN, "="),
            (INT, "5"),
            (SEMICOLON, ";"),

            (LET, "let"),
            (IDENT, "ten"),
            (ASSIGN, "="),
            (INT, "10"),
            (SEMICOLON, ";"),

            (LET,"let"),
            (IDENT,"add"),
            (ASSIGN,"="),
            (FUNCTION,"fn"),
            (LPAREN, "("),
            (IDENT, "x"),
            (COMMA, ","),
            (IDENT, "y"),
            (RPAREN, ")"),
            (LBRACE, "{"),
            (IDENT, "x"),
            (PLUS, "+"),
            (IDENT, "y"),
            (SEMICOLON, ";"),
            (RBRACE, "}"),
            (SEMICOLON, ";"),

            (LET, "let"),
            (IDENT, "result"),
            (ASSIGN, "="),
            (IDENT, "add"),
            (LPAREN, "("),
            (IDENT, "five"),
            (COMMA, ","),
            (IDENT, "ten"),
            (RPAREN, ")"),
            (SEMICOLON, ";"),

            (BANG,"!"),
            (MINUS,"-"),
            (SLASH,"/"),
            (ASTERISK,"*"),
            (INT,"5"),
            (SEMICOLON,";"),

            (INT,"5"),
            (LT,"<"),
            (INT,"10"),
            (GT,">"),
            (INT,"5"),
            (SEMICOLON,";"),

            (IF,"if"),
            (LPAREN,"("),
            (INT,"5"),
            (LT,"<"),
            (INT,"10"),
            (RPAREN,")"),
            (LBRACE,"{"),
            (RETURN,"return"),
            (TRUE,"true"),
            (SEMICOLON,";"),
            (RBRACE,"}"),
            (ELSE,"else"),
            (LBRACE,"{"),
            (RETURN,"return"),
            (FALSE,"false"),
            (SEMICOLON,";"),
            (RBRACE,"}"),

            (INT, "10"),
            (EQ, "=="),
            (INT, "10"),
            (SEMICOLON,";"),

            (INT, "10"),
            (NOT_EQ, "!="),
            (INT, "9"),
            (SEMICOLON,";"),

            (EOF, ""),
            ];

        let mut lexer = Lexer::new(input);
        for i in 0..tests.len(){
            let token = lexer.next_token();
            assert_eq!(token.type_f, tests[i].0, "incorrect token type: {}",i);
            assert_eq!(token.literal, tests[i].1, "incorrect token litera: {}",i);
        }
    }
}
