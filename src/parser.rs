#![allow(dead_code)]

use std::collections::HashMap;

use crate::ast;
use crate::lexer;
use crate::token;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum OperatorPrecedence {
    LOWEST = 1,
    EQUALS,      // ==
    LESSGREATER, // > or <
    SUM,         // +
    PRODUCT,     // *
    PREFIX,      // -X or !X
    CALL,        // myFunction(X)
}

type PrefixParseFn = fn(&mut Parser) -> Option<ast::BoxedExpression>;
type InfixParseFn = fn(&mut Parser, ast::BoxedExpression) -> Option<ast::BoxedExpression>;

pub struct Parser<'a> {
    lexer: lexer::Lexer<'a>,
    errors: Vec<String>,
    current_token: token::Token,
    peek_token: token::Token,
    prefix_parse_fns: HashMap<token::TokenType, PrefixParseFn>,
    infix_parse_fns: HashMap<token::TokenType, InfixParseFn>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: lexer::Lexer<'a>) -> Self {
        let mut p: Parser<'_> = Parser {
            lexer,
            current_token: token::Token::new_empty(),
            peek_token: token::Token::new_empty(),
            errors: Vec::new(),
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };

        // registrar
        p.register_prefix(token::IDENT.to_string(), Parser::parse_identifier);
        p.register_prefix(token::INT.to_string(), Parser::parse_integer_literal);

        p.register_prefix(token::BANG.to_string(), Parser::parse_prefix_expression);
        p.register_prefix(token::MINUS.to_string(), Parser::parse_prefix_expression);

        // Initialize peek_token and current_token
        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn current_token_is(&self, token_type: token::TokenType) -> bool {
        return self.current_token.toke_type == token_type;
    }

    fn peek_token_is(&self, token_type: token::TokenType) -> bool {
        return self.peek_token.toke_type == token_type;
    }

    fn expect_peek(&mut self, token_type: token::TokenType) -> bool {
        if self.peek_token_is(token_type.clone()) {
            self.next_token();
            return true;
        } else {
            self.peek_error(token_type);
            return false;
        }
    }

    pub fn get_errors(&self) -> Vec<String> {
        return self.errors.clone();
    }

    fn peek_error(&mut self, token_type: token::TokenType) {
        let msg: String = format!(
            "expected next token to be \"{}\", got \"{}\" instead",
            token_type, self.peek_token.toke_type
        );
        self.errors.push(msg);
    }

    fn no_prefix_parse_fn_error(&mut self, token_type: token::TokenType) {
        self.errors
            .push(format!("no prefix parse function for {} found", token_type))
    }

    pub fn parse_program(&mut self) -> Option<ast::Program> {
        let mut program: ast::Program = ast::Program {
            statements: Vec::new(),
        };

        while self.current_token.toke_type != token::EOF {
            let statement_opt: Option<ast::BoxedStatement> = self.parse_statement();
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

    pub fn parse_statement(&mut self) -> Option<ast::BoxedStatement> {
        match self.current_token.toke_type.as_str() {
            token::LET => self.parse_let_statement(),
            token::RETURN => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<ast::BoxedStatement> {
        let mut statement: Box<ast::LetStatement> = Box::new(ast::LetStatement {
            token: self.current_token.clone(),
            value: ast::Identifier::new_empty(),
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

    fn parse_return_statement(&mut self) -> Option<ast::BoxedStatement> {
        let statement: Box<ast::ReturnStatement> = Box::new(ast::ReturnStatement {
            token: self.current_token.clone(),
            return_value: None,
        });
        self.next_token();
        while !self.current_token_is(token::SEMICOLON.to_string()) {
            self.next_token();
        }
        return Some(statement);
    }

    fn parse_expression_statement(&mut self) -> Option<ast::BoxedStatement> {
        let statement: Box<ast::ExpressionStatement> = Box::new(ast::ExpressionStatement {
            token: self.current_token.clone(),
            expression: self.parse_expression(OperatorPrecedence::LOWEST),
        });

        if self.peek_token_is(token::SEMICOLON.to_string()) {
            self.next_token();
        }
        Some(statement)
    }
    fn parse_expression(&mut self, precedence: OperatorPrecedence) -> Option<ast::BoxedExpression> {
        let prefix_opt: Option<&PrefixParseFn> = self
            .prefix_parse_fns
            .get(self.current_token.toke_type.as_str());
        match prefix_opt {
            Some(func) => func(self),
            None => {
                self.no_prefix_parse_fn_error(self.current_token.toke_type.clone());
                None
            }
        }
    }

    fn parse_identifier(p: &mut Parser) -> Option<ast::BoxedExpression> {
        Some(Box::new(ast::Identifier {
            token: p.current_token.clone(),
            value: p.current_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(p: &mut Parser) -> Option<ast::BoxedExpression> {
        match p.current_token.literal.parse::<i64>() {
            Ok(value) => Some(Box::new(ast::IntegerLiteral {
                token: p.current_token.clone(),
                value,
            })),
            Err(e) => {
                p.errors.push(format!(
                    "could not parse {} as integer, {}",
                    p.current_token.literal, e
                ));
                None
            }
        }
    }

    fn parse_prefix_expression(p: &mut Parser) -> Option<ast::BoxedExpression> {
        let mut expression = Box::new(ast::PrefixExpression {
            token: p.current_token.clone(),
            operator: p.current_token.literal.clone(),
            right: None,
        });
        p.next_token();
        expression.right = p.parse_expression(OperatorPrecedence::PREFIX);
        Some(expression)
    }

    fn register_prefix(&mut self, token_type: token::TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, func);
    }

    fn register_infix(&mut self, token_type: token::TokenType, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, func);
    }
}

#[cfg(test)]
mod tests {

    use crate::ast;
    use crate::ast::Node;
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
        let program: ast::Program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 3);
        for i in 0..identifiers.len() {
            let expected_identifier = identifiers[i];
            let generic_statement: &ast::BoxedStatement = &program.statements[i];

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

    #[test]
    fn test_return_statement() {
        let input = "
            return 5;
            return 10;
            return 993322;
            ";
        let lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program().expect("Error parsing program");
        assert_eq!(
            program.statements.len(),
            3,
            "program.Statements does not contain 3 statements. got={}",
            program.statements.len()
        );

        for i in 0..program.statements.len() {
            let generic_statement: &ast::BoxedStatement = &program.statements[i];

            assert_eq!(generic_statement.token_literal(), "return");

            // "type assertion"
            let return_statement_opt: Option<&ast::ReturnStatement> = generic_statement
                .as_any()
                .downcast_ref::<ast::ReturnStatement>(
            );

            match return_statement_opt {
                Some(statement) => {
                    assert_eq!(
                        statement.token_literal(),
                        "return",
                        "statement.token_literal not \"return\", got=\"{}\"",
                        statement.token_literal()
                    );
                }
                None => {
                    assert!(false, "statement not &ReturnStatement")
                }
            }
        }
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";
        let lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program().expect("Error parsing program");

        assert_eq!(
            program.statements.len(),
            1,
            "program has not enough statements, got={}",
            program.statements.len()
        );

        let statement: &ast::BoxedStatement = &program.statements[0];

        let expression_statement: &ast::ExpressionStatement = match statement
            .as_any()
            .downcast_ref::<ast::ExpressionStatement>(
        ) {
            Some(expr_stmt) => expr_stmt,
            None => panic!("program.statements[0] is not an ast::ExpressionStatement"),
        };

        let expression: &ast::BoxedExpression = match expression_statement.expression.as_ref() {
            Some(expr) => expr,
            None => panic!("no expression in ast::ExpressionStatement"),
        };

        let identifier: &ast::Identifier =
            match expression.as_any().downcast_ref::<ast::Identifier>() {
                Some(id) => id,
                None => panic!("expression is not ast::Identifier"),
            };

        assert_eq!(
            identifier.value, "foobar",
            "identifier.value not \"foobar\", got={}",
            identifier.value
        );

        assert_eq!(
            identifier.token_literal(),
            "foobar",
            "identifier.token_literal not \"foobar\", got={}",
            identifier.token_literal()
        );
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";
        let lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program().expect("Error parsing program");

        assert_eq!(
            program.statements.len(),
            1,
            "program has not enough statements, got={}",
            program.statements.len()
        );

        let statement: &ast::BoxedStatement = &program.statements[0];

        let expression_statement: &ast::ExpressionStatement = match statement
            .as_any()
            .downcast_ref::<ast::ExpressionStatement>(
        ) {
            Some(expr_stmt) => expr_stmt,
            None => panic!("program.statements[0] is not an ast::ExpressionStatement"),
        };

        let expression: &ast::BoxedExpression = match expression_statement.expression.as_ref() {
            Some(expr) => expr,
            None => panic!("no expression in ast::ExpressionStatement"),
        };

        let literal: &ast::IntegerLiteral =
            match expression.as_any().downcast_ref::<ast::IntegerLiteral>() {
                Some(id) => id,
                None => panic!("expression is not ast::IntegerLiteral"),
            };

        assert_eq!(
            literal.value, 5,
            "literal.value no int {}, got {}",
            5, literal.value
        );
        assert_eq!(
            literal.token_literal(),
            "5",
            "literal.value no string \"{}\", got \"{}\"",
            "5",
            literal.token_literal()
        );
    }
}
