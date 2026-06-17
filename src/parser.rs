#![allow(dead_code)]

use std::collections::HashMap;

use crate::ast;
use crate::lexer;
use crate::token::{self, TokenType};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Precedence {
    LOWEST = 1,
    EQUALS,      // ==
    LESSGREATER, // > or <
    SUM,         // +
    PRODUCT,     // *
    PREFIX,      // -X or !X
    CALL,        // MYfunction(X)
}

fn precedences(token_type: TokenType) -> Precedence {
    match token_type {
        TokenType::Eq | TokenType::NotEq => Precedence::EQUALS,
        TokenType::Lt | TokenType::Gt => Precedence::LESSGREATER,
        TokenType::Plus | TokenType::Minus => Precedence::SUM,
        TokenType::Slash | TokenType::Asterisk => Precedence::PRODUCT,
        TokenType::Lparen => Precedence::CALL,
        _ => Precedence::LOWEST,
    }
}

type PrefixParseFn<'a> = fn(&mut Parser<'a>) -> Option<ast::Expression<'a>>;
type InfixParseFn<'a> = fn(&mut Parser<'a>, ast::Expression<'a>) -> Option<ast::Expression<'a>>;

pub struct Parser<'a> {
    lexer: lexer::Lexer<'a>,
    errors: Vec<String>,
    current_token: token::Token<'a>,
    peek_token: token::Token<'a>,
    prefix_parse_fns: HashMap<token::TokenType, PrefixParseFn<'a>>,
    infix_parse_fns: HashMap<token::TokenType, InfixParseFn<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: lexer::Lexer<'a>) -> Self {
        let mut p = Parser {
            lexer,
            current_token: token::Token::new(TokenType::Illegal, ""),
            peek_token: token::Token::new(TokenType::Illegal, ""),
            errors: Vec::new(),
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };

        p.register_prefix(TokenType::Ident, Parser::parse_identifier);
        p.register_prefix(TokenType::Int, Parser::parse_integer_literal);
        p.register_prefix(TokenType::Bang, Parser::parse_prefix_expression);
        p.register_prefix(TokenType::Minus, Parser::parse_prefix_expression);
        p.register_prefix(TokenType::If, Parser::parse_if_expression);
        p.register_prefix(TokenType::False, Parser::parse_boolean_expression);
        p.register_prefix(TokenType::True, Parser::parse_boolean_expression);
        p.register_prefix(TokenType::Lparen, Parser::parse_grouped_expression);
        p.register_prefix(TokenType::Function, Parser::parse_function_literal);
        p.register_prefix(TokenType::String, Parser::parse_string_literal);

        p.register_infix(TokenType::Plus, Parser::parse_infix_expression);
        p.register_infix(TokenType::Minus, Parser::parse_infix_expression);
        p.register_infix(TokenType::Slash, Parser::parse_infix_expression);
        p.register_infix(TokenType::Asterisk, Parser::parse_infix_expression);
        p.register_infix(TokenType::Eq, Parser::parse_infix_expression);
        p.register_infix(TokenType::NotEq, Parser::parse_infix_expression);
        p.register_infix(TokenType::Lt, Parser::parse_infix_expression);
        p.register_infix(TokenType::Gt, Parser::parse_infix_expression);

        p.register_infix(TokenType::Lparen, Parser::parse_call_expression);

        // Initialize peek_token and current_token
        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token;
        self.peek_token = self.lexer.next_token();
    }

    fn current_token_is(&self, token_type: TokenType) -> bool {
        self.current_token.toke_type == token_type
    }

    fn peek_token_is(&self, token_type: TokenType) -> bool {
        self.peek_token.toke_type == token_type
    }

    fn expect_peek(&mut self, token_type: TokenType) -> bool {
        if self.peek_token_is(token_type) {
            self.next_token();
            true
        } else {
            self.peek_error(token_type);
            false
        }
    }

    pub fn get_errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    fn peek_error(&mut self, token_type: TokenType) {
        let msg = format!(
            "expected next token to be \"{}\", got \"{}\" instead",
            token_type, self.peek_token.toke_type
        );
        self.errors.push(msg);
    }

    fn no_prefix_parse_fn_error(&mut self, token_type: TokenType) {
        self.errors
            .push(format!("no prefix parse function for {} found", token_type))
    }

    pub fn parse_program(&mut self) -> Option<ast::Program<'a>> {
        let mut program = ast::Program {
            statements: Vec::new(),
        };

        while self.current_token.toke_type != TokenType::Eof {
            if let Some(statement) = self.parse_statement() {
                program.statements.push(statement);
            }
            self.next_token();
        }
        Some(program)
    }

    fn parse_statement(&mut self) -> Option<ast::Statement<'a>> {
        match self.current_token.toke_type {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<ast::Statement<'a>> {
        let let_token = self.current_token;

        if !self.expect_peek(TokenType::Ident) {
            return None;
        }

        let name = ast::Identifier {
            token: self.current_token,
            value: self.current_token.literal,
        };

        if !self.expect_peek(TokenType::Assign) {
            return None;
        }

        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST)?;

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ast::Statement::Let(ast::LetStatement {
            token: let_token,
            name,
            value,
        }))
    }

    fn parse_return_statement(&mut self) -> Option<ast::Statement<'a>> {
        let token = self.current_token;
        self.next_token();

        let return_value = self.parse_expression(Precedence::LOWEST).map(Box::new);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ast::Statement::Return(ast::ReturnStatement {
            token,
            return_value,
        }))
    }

    fn parse_expression_statement(&mut self) -> Option<ast::Statement<'a>> {
        let token = self.current_token;
        let expression = self.parse_expression(Precedence::LOWEST)?;
        let stmt = ast::ExpressionStatement {
            token,
            expression: Box::new(expression),
        };

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }
        Some(ast::Statement::Expression(stmt))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<ast::Expression<'a>> {
        let prefix_fn = self
            .prefix_parse_fns
            .get(&self.current_token.toke_type)
            .cloned();

        let mut left_exp = match prefix_fn {
            Some(p_fn) => p_fn(self)?,
            None => {
                self.no_prefix_parse_fn_error(self.current_token.toke_type);
                return None;
            }
        };

        while !self.peek_token_is(TokenType::Semicolon) && precedence < self.peek_precedence() {
            let infix_fn = self
                .infix_parse_fns
                .get(&self.peek_token.toke_type)
                .cloned();

            match infix_fn {
                Some(i_fn) => {
                    self.next_token();
                    left_exp = i_fn(self, left_exp)?;
                }
                None => return Some(left_exp),
            }
        }

        Some(left_exp)
    }

    fn parse_identifier(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        Some(ast::Expression::Identifier(ast::Identifier {
            token: p.current_token,
            value: p.current_token.literal,
        }))
    }

    fn parse_integer_literal(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        match p.current_token.literal.parse::<i64>() {
            Ok(value) => Some(ast::Expression::IntegerLiteral(ast::IntegerLiteral {
                token: p.current_token,
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

    fn parse_prefix_expression(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        let token = p.current_token;
        let operator = p.current_token.literal;

        p.next_token();

        let right = p.parse_expression(Precedence::PREFIX)?;

        Some(ast::Expression::Prefix(ast::PrefixExpression {
            token,
            operator,
            right: Box::new(right),
        }))
    }

    fn parse_infix_expression(
        p: &mut Parser<'a>,
        left: ast::Expression<'a>,
    ) -> Option<ast::Expression<'a>> {
        let token = p.current_token;
        let operator = p.current_token.literal;
        let precedence = p.current_precedence();

        p.next_token();

        let right = p.parse_expression(precedence)?;

        Some(ast::Expression::Infix(ast::InfixExpression {
            token,
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }))
    }

    fn parse_grouped_expression(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        p.next_token();
        let expression = p.parse_expression(Precedence::LOWEST);
        if !p.expect_peek(TokenType::Rparen) {
            return None;
        }
        expression
    }

    fn parse_boolean_expression(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        Some(ast::Expression::Boolean(ast::Boolean {
            token: p.current_token,
            value: p.current_token_is(TokenType::True),
        }))
    }

    fn parse_if_expression(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        let token = p.current_token;

        if !p.expect_peek(TokenType::Lparen) {
            return None;
        }
        p.next_token();

        let condition = p.parse_expression(Precedence::LOWEST)?;

        if !p.expect_peek(TokenType::Rparen) {
            return None;
        }

        if !p.expect_peek(TokenType::Lbrace) {
            return None;
        }

        let consequence = p.parse_block_statement();

        let alternative = if p.peek_token_is(TokenType::Else) {
            p.next_token();
            if !p.expect_peek(TokenType::Lbrace) {
                return None;
            }
            Some(p.parse_block_statement())
        } else {
            None
        };

        Some(ast::Expression::If(ast::IfExpression {
            token,
            condition: Box::new(condition),
            consequence,
            alternative,
        }))
    }

    fn parse_function_literal(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        let token = p.current_token;
        if !p.expect_peek(TokenType::Lparen) {
            return None;
        }

        let parameters = p.parse_function_parameters();

        if !p.expect_peek(TokenType::Lbrace) {
            return None;
        }

        let body = p.parse_block_statement();

        Some(ast::Expression::FunctionLiteral(ast::FunctionLiteral {
            token,
            parameters,
            body,
        }))
    }

    fn parse_string_literal(p: &mut Parser<'a>) -> Option<ast::Expression<'a>> {
        Some(ast::Expression::StringLiteral(ast::StringLiteral{
            token: p.current_token,
            value: p.current_token.literal.to_string(),
        }))
    }

    fn parse_function_parameters(&mut self) -> Vec<ast::Identifier<'a>> {
        let mut identifiers = Vec::new();
        if self.peek_token_is(TokenType::Rparen) {
            self.next_token();
            return identifiers;
        }
        self.next_token();
        identifiers.push(ast::Identifier {
            token: self.current_token,
            value: self.current_token.literal,
        });
        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            identifiers.push(ast::Identifier {
                token: self.current_token,
                value: self.current_token.literal,
            });
        }
        if !self.expect_peek(TokenType::Rparen) {
            return Vec::new(); // Or handle error
        }
        identifiers
    }

    fn parse_call_expression(
        p: &mut Parser<'a>,
        function: ast::Expression<'a>,
    ) -> Option<ast::Expression<'a>> {
        let token = p.current_token;
        let arguments = p.parse_call_arguments()?;
        Some(ast::Expression::Call(ast::CallExpression {
            token,
            function: Box::new(function),
            arguments,
        }))
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<ast::Expression<'a>>> {
        let mut arguments = Vec::new();

        if self.peek_token_is(TokenType::Rparen) {
            self.next_token();
            return Some(arguments);
        }

        self.next_token();
        arguments.push(self.parse_expression(Precedence::LOWEST)?);

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            arguments.push(self.parse_expression(Precedence::LOWEST)?);
        }

        if !self.expect_peek(TokenType::Rparen) {
            return None;
        }

        Some(arguments)
    }

    fn parse_block_statement(&mut self) -> ast::BlockStatement<'a> {
        let mut block = ast::BlockStatement {
            token: self.current_token,
            statements: Vec::new(),
        };

        self.next_token();

        while !self.current_token_is(TokenType::Rbrace) && !self.current_token_is(TokenType::Eof) {
            if let Some(statement) = self.parse_statement() {
                block.statements.push(statement)
            }
            self.next_token();
        }

        block
    }

    fn register_prefix(&mut self, token_type: TokenType, func: PrefixParseFn<'a>) {
        self.prefix_parse_fns.insert(token_type, func);
    }

    fn register_infix(&mut self, token_type: TokenType, func: InfixParseFn<'a>) {
        self.infix_parse_fns.insert(token_type, func);
    }

    fn peek_precedence(&self) -> Precedence {
        precedences(self.peek_token.toke_type)
    }

    fn current_precedence(&self) -> Precedence {
        precedences(self.current_token.toke_type)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::lexer;
    use crate::parser::Parser;

    fn check_parser_errors(parser: &Parser) {
        let errors = parser.get_errors();
        if errors.is_empty() {
            return;
        }
        let mut msg = format!("parser has {} errors\n", errors.len());
        for err in errors.iter() {
            msg.push_str(&format!("parser error: {}\n", err));
        }
        panic!("{}", msg);
    }

    #[derive(Clone, Debug)]
    enum PrimitiveValue<'a> {
        Int64(i64),
        Str(&'a str),
        Bool(bool),
    }

    fn test_integer_literal(expr: &ast::Expression, expected_value: i64) {
        if let ast::Expression::IntegerLiteral(int_lit) = expr {
            assert_eq!(int_lit.value, expected_value);
            assert_eq!(int_lit.token.literal, expected_value.to_string());
        } else {
            panic!("Expression is not an IntegerLiteral. Got: {:?}", expr);
        }
    }

    fn test_identifier(expr: &ast::Expression, expected_value: &str) {
        if let ast::Expression::Identifier(ident) = expr {
            assert_eq!(ident.value, expected_value);
            assert_eq!(ident.token.literal, expected_value);
        } else {
            panic!("Expression is not an Identifier. Got: {:?}", expr);
        }
    }

    fn test_boolean_literal(expr: &ast::Expression, expected_value: bool) {
        if let ast::Expression::Boolean(b) = expr {
            assert_eq!(b.value, expected_value);
            assert_eq!(b.token.literal, expected_value.to_string());
        } else {
            panic!("Expression is not a Boolean. Got: {:?}", expr);
        }
    }

    fn test_literal_expression(expr: &ast::Expression, expected: PrimitiveValue) {
        match expected {
            PrimitiveValue::Int64(v) => test_integer_literal(expr, v),
            PrimitiveValue::Str(v) => test_identifier(expr, v),
            PrimitiveValue::Bool(v) => test_boolean_literal(expr, v),
        }
    }

    fn test_infix_expression(
        expr: &ast::Expression,
        expected_left: PrimitiveValue,
        expected_op: &str,
        expected_right: PrimitiveValue,
    ) {
        if let ast::Expression::Infix(infix_expr) = expr {
            test_literal_expression(&infix_expr.left, expected_left);
            assert_eq!(infix_expr.operator, expected_op);
            test_literal_expression(&infix_expr.right, expected_right);
        } else {
            panic!("Expression is not an InfixExpression. Got: {:?}", expr);
        }
    }

    #[test]
    fn test_let_statements() {
        let input = "
            let x = 5;
            let y = 10;
            let foobar = 838383;
        ";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser
            .parse_program()
            .expect("parse_program() returned None");
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 3);

        let expected_identifiers = ["x", "y", "foobar"];
        for (i, stmt) in program.statements.iter().enumerate() {
            if let ast::Statement::Let(let_stmt) = stmt {
                assert_eq!(let_stmt.name.value, expected_identifiers[i]);
            } else {
                panic!("Statement is not a LetStatement. Got: {:?}", stmt);
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let input = "
            return 5;
            return 10;
            return 993322;
        ";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 3);

        for stmt in program.statements.iter() {
            assert!(matches!(stmt, ast::Statement::Return(_)));
        }
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];

        if let ast::Statement::Expression(expr_stmt) = stmt {
            test_identifier(&expr_stmt.expression, "foobar");
        } else {
            panic!("Statement is not an ExpressionStatement. Got: {:?}", stmt);
        }
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];

        if let ast::Statement::Expression(expr_stmt) = stmt {
            test_integer_literal(&expr_stmt.expression, 5);
        } else {
            panic!("Statement is not an ExpressionStatement. Got: {:?}", stmt);
        }
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let prefix_tests = [
            ("!5;", "!", PrimitiveValue::Int64(5)),
            ("-15;", "-", PrimitiveValue::Int64(15)),
            ("!true;", "!", PrimitiveValue::Bool(true)),
            ("!false;", "!", PrimitiveValue::Bool(false)),
        ];

        for (input, operator, value) in prefix_tests.iter() {
            let lexer = lexer::Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);
            let stmt = &program.statements[0];

            if let ast::Statement::Expression(expr_stmt) = stmt {
                if let ast::Expression::Prefix(prefix_expr) = &*expr_stmt.expression {
                    assert_eq!(prefix_expr.operator, *operator);
                    test_literal_expression(&prefix_expr.right, value.clone());
                } else {
                    panic!(
                        "Expression is not a PrefixExpression. Got: {:?}",
                        expr_stmt.expression
                    );
                }
            } else {
                panic!("Statement is not an ExpressionStatement. Got: {:?}", stmt);
            }
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        let infix_tests = [
            (
                "5 + 5;",
                PrimitiveValue::Int64(5),
                "+",
                PrimitiveValue::Int64(5),
            ),
            (
                "5 - 5;",
                PrimitiveValue::Int64(5),
                "-",
                PrimitiveValue::Int64(5),
            ),
            (
                "5 * 5;",
                PrimitiveValue::Int64(5),
                "*",
                PrimitiveValue::Int64(5),
            ),
            (
                "5 / 5;",
                PrimitiveValue::Int64(5),
                "/",
                PrimitiveValue::Int64(5),
            ),
            (
                "5 > 5;",
                PrimitiveValue::Int64(5),
                ">",
                PrimitiveValue::Int64(5),
            ),
            (
                "5 < 5;",
                PrimitiveValue::Int64(5),
                "<",
                PrimitiveValue::Int64(5),
            ),
            (
                "5 == 5;",
                PrimitiveValue::Int64(5),
                "==",
                PrimitiveValue::Int64(5),
            ),
            (
                "5 != 5;",
                PrimitiveValue::Int64(5),
                "!=",
                PrimitiveValue::Int64(5),
            ),
            (
                "true == true",
                PrimitiveValue::Bool(true),
                "==",
                PrimitiveValue::Bool(true),
            ),
            (
                "true != false",
                PrimitiveValue::Bool(true),
                "!=",
                PrimitiveValue::Bool(false),
            ),
            (
                "false == false",
                PrimitiveValue::Bool(false),
                "==",
                PrimitiveValue::Bool(false),
            ),
        ];

        for (input, left_val, op, right_val) in infix_tests.iter() {
            let lexer = lexer::Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);
            let stmt = &program.statements[0];

            if let ast::Statement::Expression(expr_stmt) = stmt {
                test_infix_expression(
                    &expr_stmt.expression,
                    left_val.clone(),
                    op,
                    right_val.clone(),
                );
            } else {
                panic!("Statement is not an ExpressionStatement. Got: {:?}", stmt);
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = [
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("true", "true"),
            ("false", "false"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
            ("a + add(b * c) + d", "((a + add((b * c))) + d)"),
            (
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            ),
            (
                "add(a + b + c * d / f + g)",
                "add((((a + b) + ((c * d) / f)) + g))",
            ),
        ];

        for (input, expected) in tests.iter() {
            let lexer = lexer::Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            check_parser_errors(&parser);
            assert_eq!(program.to_string(), *expected);
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];

        if let ast::Statement::Expression(expr_stmt) = stmt {
            if let ast::Expression::If(if_expr) = &*expr_stmt.expression {
                test_infix_expression(
                    &if_expr.condition,
                    PrimitiveValue::Str("x"),
                    "<",
                    PrimitiveValue::Str("y"),
                );
                assert_eq!(if_expr.consequence.statements.len(), 1);
                if let ast::Statement::Expression(cons_expr_stmt) =
                    &if_expr.consequence.statements[0]
                {
                    test_identifier(&cons_expr_stmt.expression, "x");
                } else {
                    panic!("Consequence statement is not an ExpressionStatement.");
                }
                assert!(if_expr.alternative.is_none());
            } else {
                panic!("Expression is not an IfExpression.");
            }
        } else {
            panic!("Statement is not an ExpressionStatement.");
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];

        if let ast::Statement::Expression(expr_stmt) = stmt {
            if let ast::Expression::If(if_expr) = &*expr_stmt.expression {
                test_infix_expression(
                    &if_expr.condition,
                    PrimitiveValue::Str("x"),
                    "<",
                    PrimitiveValue::Str("y"),
                );

                assert_eq!(if_expr.consequence.statements.len(), 1);
                if let ast::Statement::Expression(cons_expr_stmt) =
                    &if_expr.consequence.statements[0]
                {
                    test_identifier(&cons_expr_stmt.expression, "x");
                } else {
                    panic!("Consequence statement is not an ExpressionStatement.");
                }

                assert!(if_expr.alternative.is_some());
                let alt_block = if_expr.alternative.as_ref().unwrap();
                assert_eq!(alt_block.statements.len(), 1);
                if let ast::Statement::Expression(alt_expr_stmt) = &alt_block.statements[0] {
                    test_identifier(&alt_expr_stmt.expression, "y");
                } else {
                    panic!("Alternative statement is not an ExpressionStatement.");
                }
            } else {
                panic!("Expression is not an IfExpression.");
            }
        } else {
            panic!("Statement is not an ExpressionStatement.");
        }
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = "fn(x, y) { x + y; }";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];

        if let ast::Statement::Expression(expr_stmt) = stmt {
            if let ast::Expression::FunctionLiteral(func_lit) = &*expr_stmt.expression {
                assert_eq!(func_lit.parameters.len(), 2);
                assert_eq!(func_lit.parameters[0].value, "x");
                assert_eq!(func_lit.parameters[1].value, "y");

                assert_eq!(func_lit.body.statements.len(), 1);
                if let ast::Statement::Expression(body_expr_stmt) = &func_lit.body.statements[0] {
                    test_infix_expression(
                        &body_expr_stmt.expression,
                        PrimitiveValue::Str("x"),
                        "+",
                        PrimitiveValue::Str("y"),
                    );
                } else {
                    panic!("Function body statement is not an ExpressionStatement.");
                }
            } else {
                panic!("Expression is not a FunctionLiteral.");
            }
        } else {
            panic!("Statement is not an ExpressionStatement.");
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);";
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];

        if let ast::Statement::Expression(expr_stmt) = stmt {
            if let ast::Expression::Call(call_expr) = &*expr_stmt.expression {
                test_identifier(&call_expr.function, "add");
                assert_eq!(call_expr.arguments.len(), 3);
                test_literal_expression(&call_expr.arguments[0], PrimitiveValue::Int64(1));
                test_infix_expression(
                    &call_expr.arguments[1],
                    PrimitiveValue::Int64(2),
                    "*",
                    PrimitiveValue::Int64(3),
                );
                test_infix_expression(
                    &call_expr.arguments[2],
                    PrimitiveValue::Int64(4),
                    "+",
                    PrimitiveValue::Int64(5),
                );
            } else {
                panic!("Expression is not a CallExpression.");
            }
        } else {
            panic!("Statement is not an ExpressionStatement.");
        }
    }

    #[test]
    fn test_string_literal_expression() {
        let input = r#""hello world""#;
        let lexer = lexer::Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        if let ast::Statement::Expression(expr) = &program.statements[0]{
            if let ast::Expression::StringLiteral(str_lit)  = &*expr.expression {
                assert_eq!("hello world", str_lit.value, "str_lit.value is not \"hello world\", got\"str_lit.value\"");
            }else{
                panic!("expr.expression is not Expression::StringLiteral")
            }
        }
    }



}
