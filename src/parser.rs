#![allow(dead_code)]

use std::collections::HashMap;

use crate::ast_enum as ast;
use crate::lexer;
use crate::token;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Precedence {
    LOWEST = 1,
    EQUALS,      // ==
    LESSGREATER, // > or <
    SUM,         // +
    PRODUCT,     // *
    PREFIX,      // -X or !X
    CALL,        // MYfunction(X)
}

const PRECEDENCES_MAP: [(&str, Precedence); 9] = [
    (token::EQ, Precedence::EQUALS),
    (token::NOT_EQ, Precedence::EQUALS),
    (token::LT, Precedence::LESSGREATER),
    (token::GT, Precedence::LESSGREATER),
    (token::PLUS, Precedence::SUM),
    (token::MINUS, Precedence::SUM),
    (token::SLASH, Precedence::PRODUCT),
    (token::ASTERISK, Precedence::PRODUCT),
    (token::LPAREN, Precedence::CALL),
];

fn precedences(token_type: &str) -> Option<Precedence> {
    for p in PRECEDENCES_MAP.iter() {
        if p.0 == token_type {
            return Some(p.1.clone());
        }
    }
    None
}

type PrefixParseFn = fn(&mut Parser) -> Option<ast::Expression>;
type InfixParseFn = fn(&mut Parser, ast::Expression) -> Option<ast::Expression>;

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

        p.register_prefix(token::IDENT.to_string(), Parser::parse_identifier);
        p.register_prefix(token::INT.to_string(), Parser::parse_integer_literal);
        p.register_prefix(token::BANG.to_string(), Parser::parse_prefix_expression);
        p.register_prefix(token::MINUS.to_string(), Parser::parse_prefix_expression);
        p.register_prefix(token::IF.to_string(), Parser::parse_if_expression);
        p.register_prefix(token::FALSE.to_string(), Parser::parse_boolean_expression);
        p.register_prefix(token::TRUE.to_string(), Parser::parse_boolean_expression);
        p.register_prefix(token::LPAREN.to_string(), Parser::parse_grouped_expression);
        p.register_prefix(token::FUNCTION.to_string(), Parser::parse_function_literal);

        p.register_infix(token::PLUS.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::MINUS.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::SLASH.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::ASTERISK.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::EQ.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::NOT_EQ.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::LT.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::GT.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::LPAREN.to_string(), Parser::parse_call_expression);

        // Initialize peek_token and current_token
        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn current_token_is(&self, token_type: token::TokenType) -> bool {
        self.current_token.toke_type == token_type
    }

    fn peek_token_is(&self, token_type: token::TokenType) -> bool {
        self.peek_token.toke_type == token_type
    }

    fn expect_peek(&mut self, token_type: token::TokenType) -> bool {
        if self.peek_token_is(token_type.clone()) {
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
            if let Some(statement) = self.parse_statement() {
                program.statements.push(statement);
            }
            self.next_token();
        }
        Some(program)
    }

    fn parse_statement(&mut self) -> Option<ast::Statement> {
        match self.current_token.toke_type.as_str() {
            token::LET => self.parse_let_statement(),
            token::RETURN => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<ast::Statement> {
        let let_token: token::Token = self.current_token.clone();

        if !self.expect_peek(token::IDENT.to_string()) {
            return None;
        }

        let name: ast::Identifier = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(token::ASSIGN.to_string()) {
            return None;
        }

        self.next_token();

        let value: ast::Expression = self.parse_expression(Precedence::LOWEST)?;

        if self.peek_token_is(token::SEMICOLON.to_string()) {
            self.next_token();
        }

        Some(ast::Statement::Let(ast::LetStatement {
            token: let_token,
            name,
            value: value,
        }))
    }

    fn parse_return_statement(&mut self) -> Option<ast::Statement> {
        let stmt: ast::ReturnStatement = ast::ReturnStatement {
            token: self.current_token.clone(),
            return_value: {
                self.next_token();
                self.parse_expression(Precedence::LOWEST).map(Box::new)
            },
        };

        if self.peek_token_is(token::SEMICOLON.to_string()) {
            self.next_token();
        }

        Some(ast::Statement::Return(stmt))
    }

    fn parse_expression_statement(&mut self) -> Option<ast::Statement> {
        let expression: ast::Expression = self.parse_expression(Precedence::LOWEST)?;
        let stmt = ast::ExpressionStatement {
            token: self.current_token.clone(),
            expression: Box::new(expression),
        };

        if self.peek_token_is(token::SEMICOLON.to_string()) {
            self.next_token();
        }
        Some(ast::Statement::Expression(stmt))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<ast::Expression> {
        let prefix_fn: Option<fn(&mut Parser<'_>) -> Option<ast::Expression>> = self
            .prefix_parse_fns
            .get(self.current_token.toke_type.as_str())
            .cloned();

        let mut left_exp: ast::Expression = match prefix_fn {
            Some(p_fn) => p_fn(self)?,
            None => {
                self.no_prefix_parse_fn_error(self.current_token.toke_type.clone());
                return None;
            }
        };

        while !self.peek_token_is(token::SEMICOLON.to_string())
            && precedence < self.peek_precedence()
        {
            let infix_fn: Option<fn(&mut Parser<'_>, ast::Expression) -> Option<ast::Expression>> =
                self.infix_parse_fns
                    .get(self.peek_token.toke_type.as_str())
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

    fn parse_identifier(p: &mut Parser) -> Option<ast::Expression> {
        Some(ast::Expression::Identifier(ast::Identifier {
            token: p.current_token.clone(),
            value: p.current_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(p: &mut Parser) -> Option<ast::Expression> {
        match p.current_token.literal.parse::<i64>() {
            Ok(value) => Some(ast::Expression::IntegerLiteral(ast::IntegerLiteral {
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

    fn parse_prefix_expression(p: &mut Parser) -> Option<ast::Expression> {
        let token: token::Token = p.current_token.clone();
        let operator: String = p.current_token.literal.clone();

        p.next_token();

        let right = p.parse_expression(Precedence::PREFIX)?;

        Some(ast::Expression::Prefix(ast::PrefixExpression {
            token,
            operator,
            right: Box::new(right),
        }))
    }

    fn parse_infix_expression(p: &mut Parser, left: ast::Expression) -> Option<ast::Expression> {
        let token: token::Token = p.current_token.clone();
        let operator: String = p.current_token.literal.clone();
        let precedence: Precedence = p.current_precedence();

        p.next_token();

        let right = p.parse_expression(precedence)?;

        Some(ast::Expression::Infix(ast::InfixExpression {
            token,
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }))
    }

    fn parse_grouped_expression(p: &mut Parser) -> Option<ast::Expression> {
        p.next_token();
        let expression: Option<ast::Expression> = p.parse_expression(Precedence::LOWEST);
        if !p.expect_peek(token::RPAREN.to_string()) {
            return None;
        }
        expression
    }

    fn parse_boolean_expression(p: &mut Parser) -> Option<ast::Expression> {
        Some(ast::Expression::Boolean(ast::Boolean {
            token: p.current_token.clone(),
            value: p.current_token_is(token::TRUE.to_string()),
        }))
    }

    fn parse_if_expression(p: &mut Parser) -> Option<ast::Expression> {
        let token: token::Token = p.current_token.clone();

        if !p.expect_peek(token::LPAREN.to_string()) {
            return None;
        }
        p.next_token();

        let condition: ast::Expression = p.parse_expression(Precedence::LOWEST)?;

        if !p.expect_peek(token::RPAREN.to_string()) {
            return None;
        }

        if !p.expect_peek(token::LBRACE.to_string()) {
            return None;
        }

        let consequence: ast::BlockStatement = p.parse_block_statement();

        let alternative: Option<ast::BlockStatement> = if p.peek_token_is(token::ELSE.to_string()) {
            p.next_token();
            if !p.expect_peek(token::LBRACE.to_string()) {
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

    fn parse_function_literal(p: &mut Parser) -> Option<ast::Expression> {
        let token: token::Token = p.current_token.clone();
        if !p.expect_peek(token::LPAREN.to_string()) {
            return None;
        }

        let parameters: Vec<ast::Identifier> = p.parse_function_parameters();

        if !p.expect_peek(token::LBRACE.to_string()) {
            return None;
        }

        let body: ast::BlockStatement = p.parse_block_statement();

        Some(ast::Expression::FunctionLiteral(ast::FunctionLiteral {
            token,
            parameters,
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Vec<ast::Identifier> {
        let mut identifiers: Vec<ast::Identifier> = Vec::new();
        if self.peek_token_is(token::RPAREN.to_string()) {
            self.next_token();
            return identifiers;
        }
        self.next_token();
        identifiers.push(ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        });
        while self.peek_token_is(token::COMMA.to_string()) {
            self.next_token();
            self.next_token();
            identifiers.push(ast::Identifier {
                token: self.current_token.clone(),
                value: self.current_token.literal.clone(),
            });
        }
        if !self.expect_peek(token::RPAREN.to_string()) {
            return Vec::new(); // Or handle error
        }
        identifiers
    }

    fn parse_call_expression(p: &mut Parser, function: ast::Expression) -> Option<ast::Expression> {
        let token: token::Token = p.current_token.clone();
        let arguments: Vec<ast::Expression> = p.parse_call_arguments()?;
        Some(ast::Expression::Call(ast::CallExpression {
            token,
            function: Box::new(function),
            arguments,
        }))
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<ast::Expression>> {
        let mut arguments: Vec<ast::Expression> = Vec::new();

        if self.peek_token_is(token::RPAREN.to_string()) {
            self.next_token();
            return Some(arguments);
        }

        self.next_token();
        arguments.push(self.parse_expression(Precedence::LOWEST)?);

        while self.peek_token_is(token::COMMA.to_string()) {
            self.next_token();
            self.next_token();
            arguments.push(self.parse_expression(Precedence::LOWEST)?);
        }

        if !self.expect_peek(token::RPAREN.to_string()) {
            return None;
        }

        Some(arguments)
    }

    fn parse_block_statement(&mut self) -> ast::BlockStatement {
        let mut block: ast::BlockStatement = ast::BlockStatement {
            token: self.current_token.clone(),
            statements: Vec::new(),
        };

        self.next_token();

        while !self.current_token_is(token::RBRACE.to_string())
            && !self.current_token_is(token::EOF.to_string())
        {
            if let Some(statement) = self.parse_statement() {
                block.statements.push(statement)
            }
            self.next_token();
        }

        block
    }

    fn register_prefix(&mut self, token_type: token::TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, func);
    }

    fn register_infix(&mut self, token_type: token::TokenType, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, func);
    }

    fn peek_precedence(&self) -> Precedence {
        precedences(&self.peek_token.toke_type).unwrap_or(Precedence::LOWEST)
    }

    fn current_precedence(&self) -> Precedence {
        precedences(&self.current_token.toke_type).unwrap_or(Precedence::LOWEST)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast_enum as ast;
    use crate::lexer;
    use crate::parser::Parser;

    fn check_parser_errors(parser: &Parser) {
        let errors = parser.get_errors();
        if errors.is_empty() {
            return;
        }
        let mut msg: String = format!("parser has {} errors\n", errors.len());
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
        let input: &'static str = "foobar;";
        let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut parser: Parser<'_> = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().unwrap();
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
        let input: &'static str = "5;";
        let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut parser: Parser<'_> = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::Statement = &program.statements[0];

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
            let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
            let mut parser: Parser<'_> = Parser::new(lexer);
            let program: ast::Program = parser.parse_program().unwrap();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);
            let stmt: &ast::Statement = &program.statements[0];

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
        let infix_tests: [(
            &'static str,
            PrimitiveValue<'_>,
            &'static str,
            PrimitiveValue<'_>,
        ); 11] = [
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
            let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
            let mut parser: Parser<'_> = Parser::new(lexer);
            let program: ast::Program = parser.parse_program().unwrap();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);
            let stmt: &ast::Statement = &program.statements[0];

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
        let tests: [(&'static str, &'static str); 24] = [
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
            let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
            let mut parser: Parser<'_> = Parser::new(lexer);
            let program: ast::Program = parser.parse_program().unwrap();
            check_parser_errors(&parser);
            assert_eq!(program.to_string(), *expected);
        }
    }

    #[test]
    fn test_if_expression() {
        let input: &'static str = "if (x < y) { x }";
        let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut parser: Parser<'_> = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::Statement = &program.statements[0];

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
        let input: &'static str = "if (x < y) { x } else { y }";
        let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut parser: Parser<'_> = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().unwrap();
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
        let input: &'static str = "fn(x, y) { x + y; }";
        let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut parser: Parser<'_> = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().unwrap();
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
        let input: &'static str = "add(1, 2 * 3, 4 + 5);";
        let lexer: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut parser: Parser<'_> = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().unwrap();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::Statement = &program.statements[0];

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
}
