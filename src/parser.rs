#![allow(dead_code)]

use crate::ast;
use crate::lexer;
use crate::token;

pub struct Parser<'a> {
    pub lexer: lexer::Lexer<'a>,
    pub current_token: token::Token,
    pub peek_token: token::Token,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: lexer::Lexer<'a>) -> Self {
        let mut p: Parser<'_> = Parser {
            lexer,
            current_token: token::Token::new_empty(),
            peek_token: token::Token::new_empty(),
        };

        // Initialize peek_token and current_token
        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
        //self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }
    pub fn parse_statement(&mut self) -> Option<ast::TStatement> {
        let token_type: String = self.current_token.type_f.clone();
        match token_type.as_str() {
            token::LET => match self.parse_let_statement() {
                Some(value) => Some(value),
                None => None,
            },
            _ => return None,
        }
    }
    pub fn parse_program(&mut self) -> Option<ast::Program> {
        let mut program: ast::Program = ast::Program {
            statements: Vec::new(),
        };

        while self.current_token.type_f != token::EOF {
            let statement_opt: Option<ast::TStatement> = self.parse_statement();
            match statement_opt {
                Some(statement) => {
                    //statement.print_debug_info();
                    program.statements.push(statement);
                }
                _ => {}
            }
            self.next_token();
        }
        Some(program)
    }

    pub fn parse_let_statement(&mut self) -> Option<ast::TStatement> {
        let mut statement: Box<ast::LetStatement> = Box::new(ast::LetStatement {
            token: self.current_token.clone(),
            value: String::new(),
            name: ast::Identifier::new_empty(),
        });

        if !self.expect_peek(token::IDENT.to_string()) {
            return None;
        }

        statement.name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(token::ASSIGN.to_string()) {
            return None;
        }

        while !self.current_token_is(token::SEMICOLON.to_string()) {
            self.next_token();
        }
        Some(statement)
    }

    pub fn current_token_is(&self, token_type: token::TokenType) -> bool {
        return self.current_token.type_f == token_type;
    }

    pub fn peek_token_is(&self, token_type: token::TokenType) -> bool {
        return self.peek_token.type_f == token_type;
    }

    pub fn expect_peek(&mut self, token_type: token::TokenType) -> bool {
        if self.peek_token_is(token_type) {
            self.next_token();
            return true;
        } else {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::ast::Node;
    use crate::ast::Program;
    use crate::lexer;
    use crate::parser;

    #[test]
    fn test_let_statements() {
        let input = "
            let y = 10;
            let x = 4;
            let foobar = 838383;
            ";
        let identifiers = ["y", "x", "foobar"];
        let lexer: lexer::Lexer = lexer::Lexer::new(input);
        let mut parser: parser::Parser = parser::Parser::new(lexer);
        let program: Program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 3);
        for i in 0..identifiers.len() {
            let expected_identifier = identifiers[i];
            let generic_statement: &ast::TStatement = &program.statements[i];

            assert_eq!(generic_statement.token_literal(), "let");

            // "type assertion"
            let let_statement_opt: Option<&ast::LetStatement> = generic_statement
                .as_any()
                .downcast_ref::<ast::LetStatement>(
            );

            match let_statement_opt {
                Some(stmt) => {
                    assert_eq!(
                        stmt.name.value, expected_identifier,
                        "stmt.name.value incorrect, expected {}",
                        expected_identifier
                    );
                    assert_eq!(
                        stmt.name.token_literal(),
                        expected_identifier,
                        "stmt.name.token_literal incorrect, expected {}",
                        expected_identifier
                    );
                }
                None => panic!("the statement is NOT LetStatement"),
            }
        }
    }
}
