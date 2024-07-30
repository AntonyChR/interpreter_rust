#![allow(dead_code)]

use std::collections::HashMap;

use crate::ast;
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

const PRECEDENCES_MAP: [(&str, Precedence); 8] = [
    (token::EQ, Precedence::EQUALS),
    (token::NOT_EQ, Precedence::EQUALS),
    (token::LT, Precedence::LESSGREATER),
    (token::GT, Precedence::LESSGREATER),
    (token::PLUS, Precedence::SUM),
    (token::MINUS, Precedence::SUM),
    (token::SLASH, Precedence::PRODUCT),
    (token::ASTERISK, Precedence::PRODUCT),
];

fn precedences(token_type: String) -> Option<Precedence> {
    for p in PRECEDENCES_MAP.iter() {
        if p.0 == token_type {
            return Some(p.1.clone());
        }
    }
    return None;
}

type PrefixParseFn = fn(&mut Parser) -> Option<ast::BoxedExpression>;
type InfixParseFn = fn(&mut Parser, Option<ast::BoxedExpression>) -> Option<ast::BoxedExpression>;

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

        p.register_infix(token::PLUS.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::MINUS.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::SLASH.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::ASTERISK.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::EQ.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::NOT_EQ.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::LT.to_string(), Parser::parse_infix_expression);
        p.register_infix(token::GT.to_string(), Parser::parse_infix_expression);



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
           if let Some(statement) = self.parse_statement(){
                program.statements.push(statement);
            }
            self.next_token();
        }
        Some(program)
    }

    fn parse_statement(&mut self) -> Option<ast::BoxedStatement> {
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
            expression: self.parse_expression(Precedence::LOWEST),
        });

        if self.peek_token_is(token::SEMICOLON.to_string()) {
            self.next_token();
        }
        Some(statement)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<ast::BoxedExpression> {
        let prefix_opt: Option<&PrefixParseFn> = self
            .prefix_parse_fns
            .get(self.current_token.toke_type.as_str());

        let mut left_exp_opt: Option<ast::BoxedExpression>;

        match prefix_opt {
            Some(prefix) => {
                left_exp_opt = prefix(self);
            }
            None => {
                self.no_prefix_parse_fn_error(self.current_token.toke_type.clone());
                return None;
            }
        }

        while !self.peek_token_is(token::SEMICOLON.to_string())
            && precedence < self.peek_precedence()
        {
            // instead of cloning the entire HashMap, just clone the reference to an element
            let infix_opt:Option<InfixParseFn> = self
                .infix_parse_fns
                .get(self.peek_token.toke_type.as_str())
                .cloned();

            match infix_opt {
                Some(infix) => {
                    self.next_token();
                    left_exp_opt = infix(self, left_exp_opt);
                }
                None => {
                    return left_exp_opt;
                }
            }
        }

        return left_exp_opt;
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
        expression.right = p.parse_expression(Precedence::PREFIX);
        Some(expression)
    }

    fn parse_infix_expression(
        p: &mut Parser,
        left: Option<ast::BoxedExpression>,
    ) -> Option<ast::BoxedExpression> {
        let mut expression = Box::new(ast::InfixExpression {
            token: p.current_token.clone(),
            operator: p.current_token.literal.clone(),
            right: None,
            left,
        });

        let precedence = p.current_precedence();
        p.next_token();
        expression.right = p.parse_expression(precedence);
        Some(expression)
    }

    fn parse_grouped_expression(p: &mut Parser) -> Option<ast::BoxedExpression> {
        p.next_token();
        let expression_opt:Option<ast::BoxedExpression> = p.parse_expression(Precedence::LOWEST);
        if !p.expect_peek(token::RPAREN.to_string()){
            return None;
        }
        return expression_opt
    }

    fn parse_boolean_expression(p: &mut Parser) -> Option<ast::BoxedExpression> {
        Some(Box::new(
                ast::Boolean{
                    token: p.current_token.clone(),
                    value: p.current_token_is(token::TRUE.to_string())
                }
        ))
    }

    fn parse_if_expression(p: &mut Parser) -> Option<ast::BoxedExpression> {
        let mut expression:ast::IfExpression;

        let current_token:token::Token = p.current_token.clone();

        if !p.expect_peek(token::LPAREN.to_string()){
            return None;
        }
        p.next_token();

        let condition:ast::BoxedExpression = p.parse_expression(Precedence::LOWEST).unwrap();

        if !p.expect_peek(token::RPAREN.to_string()){
            return None;
        }

        if !p.expect_peek(token::LBRACE.to_string()){
            return None;
        }

        expression = ast::IfExpression{
                token: current_token,
                condition,
                consequence: p.parse_block_statement(),
                alternative: None
            };

        if p.peek_token_is(token::ELSE.to_string()){
            p.next_token();
            if !p.expect_peek(token::LBRACE.to_string()){
                return None;
            }
            expression.alternative = Some(p.parse_block_statement())
        }
        Some(Box::new(expression))
    }

    fn parse_block_statement(&mut self) -> ast::BlockStatement{
        let mut block = ast::BlockStatement{
            token: self.current_token.clone(),
            statements: Vec::new(),
        };

        self.next_token();

        while !self.current_token_is(token::RBRACE.to_string()) && !self.current_token_is(token::EOF.to_string()) {
            match self.parse_statement() {
                Some(statement) => block.statements.push(statement),
                _=>{}
            }
            self.next_token();
        }

        return block;

    }

    fn register_prefix(&mut self, token_type: token::TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, func);
    }

    fn register_infix(&mut self, token_type: token::TokenType, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, func);
    }

    fn peek_precedence(&self) -> Precedence {
        if let Some(p) = precedences(self.peek_token.toke_type.clone()) {
            return p;
        }
        return Precedence::LOWEST;
    }

    fn current_precedence(&self) -> Precedence {
        if let Some(p) = precedences(self.current_token.toke_type.clone()) {
            return p;
        }
        return Precedence::LOWEST;
    }
}

#[cfg(test)]
mod tests {


    use crate::ast;
    use crate::ast::Node;
    use crate::ast::Statement;
    use crate::lexer;
    use crate::parser;

    
    /////////////////////////////////////////////////////////////////////////////////////////////////
    // test helpers 

    /// panic if Parser.errors contains logged errors resulting from parsing
    fn check_parser_errors(parser: &parser::Parser) {
        let mut msg: String;

        if parser.get_errors().len() == 0 {
            return;
        }
        msg = format!("parser has {} errors\n", parser.get_errors().len());

        for err in parser.get_errors().iter() {
            msg = msg + format!("parser error: {}\n", err).as_str();
        }
        panic!("{}", msg);
    }

    /// Check if "expression" implements the ast::Identifier trait and its expect value
    fn test_identifier(expression: &ast::BoxedExpression, expected_value: String){
        let identifier_opt: Option<&ast::Identifier> = expression 
            .as_any()
            .downcast_ref::<ast::Identifier>();


        let identifier = identifier_opt.expect("ast::expression is not ast::Identifier");

        assert_eq!(
            identifier.value, 
            expected_value, 
            "identifier.value is not \"{}\", got=\"{}\"", 
            expected_value, 
            identifier.value
            );

        assert_eq!(
            identifier.token_literal(),
            expected_value,
            "identifier.token_literal is not \"{}\", got=\"{}\"",
            expected_value,
            identifier.token_literal()
        );
    }

    /// Checks that the expression if of type ast::IntegerLiteral and evaluates that
    /// ast::IntegerLiteral.value and ast::IntegerLiteral.token_literal() is equal to
    /// expected_value
    fn test_integer_literal(expression: &ast::BoxedExpression, expected_value: i64) {
        let int_literal: &ast::IntegerLiteral =
            match expression.as_any().downcast_ref::<ast::IntegerLiteral>() {
                Some(prefix_expr) => prefix_expr,
                None => panic!("expression is not ast::IntegerLiteral"),
            };

        assert_eq!(
            int_literal.value, expected_value,
            "int_literal.value not {} got={}",
            expected_value, int_literal.value
        );

        assert_eq!(
            int_literal.token_literal(),
            format!("{}", expected_value),
            "int_literal.token_literal not \"{}\" got= \"{}\"",
            expected_value,
            int_literal.token_literal()
        );
    }

    #[derive(Clone)]
    enum PrimitiveType{
        Int(i32),
        Int64(i64),
        Str(String),
        Bool(bool),
    }

    /// Checks that the literal value of the expression is equal to "expected"
    fn test_literal_expression(expression: &ast::BoxedExpression, expected: PrimitiveType){
        match expected {
            PrimitiveType::Int(v) => test_integer_literal(expression, i64::from(v)),
            PrimitiveType::Int64(v) => test_integer_literal(expression, v),
            PrimitiveType::Str(v) => test_identifier(expression, v),
            PrimitiveType::Bool(v) => test_boolean_literal(expression, v),
        }
    }

    /// Checks whether the literal values of the operands are as expected
    fn test_infix_expression(
        expression: &ast::BoxedExpression, 
        expected_left_value:PrimitiveType, 
        operator: String, 
        expected_right_value:PrimitiveType
        ){
        let operation_expression: &ast::InfixExpression=
            match expression.as_any().downcast_ref::<ast::InfixExpression>() {
                Some(op_exp) =>op_exp,
                None => panic!("expression is not ast::InfixExpression"),
            };

        let left_expression:&ast::BoxedExpression = match &operation_expression.left {
            Some(left_exp) => left_exp,
            None => panic!("no value in operation_expression.left")
            
        };

        let right_expression:&ast::BoxedExpression = match &operation_expression.right{
            Some(right_exp) => right_exp,
            None => panic!("no value in operation_expression.right")
            
        };
 
        test_literal_expression(left_expression, expected_left_value);
        assert_eq!(operation_expression.operator, operator,"expression.operator is not {}, got {}", operator, operation_expression.operator);
        test_literal_expression(right_expression, expected_right_value);
    }

    fn test_boolean_literal(expression: &ast::BoxedExpression, expected_value: bool){
        let boolean: &ast::Boolean =
            match expression.as_any().downcast_ref::<ast::Boolean>() {
                Some(prefix_expr) => prefix_expr,
                None => panic!("expression is not ast::Boolean"),
            };

        assert_eq!(
            boolean.value,
            expected_value,
            "boolean.value is not {}, got={}",
            expected_value,
            boolean.value
        );

        let expected_string = format!("{}", expected_value);

        assert_eq!(
            boolean.token_literal(),
            expected_string,
            "boolean.value is not {}, got={}",
            expected_string,
            boolean.value
        );
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // tests

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
        let program: ast::Program = parser.parse_program().expect("Error parsing program");

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
        check_parser_errors(&parser);

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
                    statement.print_debug_info();
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
        check_parser_errors(&parser);

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

        test_identifier(expression, "foobar".to_string());
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";
        let lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program().expect("Error parsing program");
        check_parser_errors(&parser);

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

        test_integer_literal(expression, 5);
    }

    #[test]
    fn test_parsing_prefix_expression() {
        struct TC<'a>{
            input: &'a str,
            operator: &'a str,
            value: PrimitiveType,
        }

        let test_cases = [
            TC{input:"!5", operator: "!", value: PrimitiveType::Int64(5)},
            TC{input:"-15", operator: "-", value: PrimitiveType::Int64(15)},
            TC{input:"!true", operator: "!", value: PrimitiveType::Bool(true)},
            TC{input:"!false", operator: "!", value: PrimitiveType::Bool(false)},
        ];

        for tc in test_cases.iter() {
            let lexer = lexer::Lexer::new(tc.input);
            let mut parser = parser::Parser::new(lexer);
            let program = parser.parse_program().expect("error parcing program");
            check_parser_errors(&parser);

            assert_eq!(
                program.statements.len(),
                1,
                "prgram.statements does not contain {} statements. got={}",
                1,
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

            let prefix_expr: &ast::PrefixExpression =
                match expression.as_any().downcast_ref::<ast::PrefixExpression>() {
                    Some(prefix_expr) => prefix_expr,
                    None => panic!("expression is not ast::PrefixExpression"),
                };

            assert_eq!(
                prefix_expr.operator, tc.operator,
                "prefix_expr.operator is not \"{}\",
                got \"{}\"",
                tc.operator, prefix_expr.operator
            );

            match &prefix_expr.right {
                Some(value) => test_literal_expression(value, tc.value.clone()),
                None => panic!("prefix_expr.righ is None"),
            };
        }
    }
    #[test]
    fn test_parsing_infix_expressions() {
        // test case
        #[derive(Clone)]
        struct TC<'a> {
            input: &'a str,
            left_value: PrimitiveType,
            operator: &'a str,
            right_value: PrimitiveType,
        }

        let infix_tests= [
            TC { input: "4 - 5;", left_value: PrimitiveType::Int64(4), operator: "-", right_value: PrimitiveType::Int64(5)},
            TC { input: "4 + 5;", left_value: PrimitiveType::Int64(4), operator: "+", right_value: PrimitiveType::Int64(5)},
            TC { input: "4 * 5;", left_value: PrimitiveType::Int64(4), operator: "*", right_value: PrimitiveType::Int64(5)},
            TC { input: "4 / 5;", left_value: PrimitiveType::Int64(4), operator: "/", right_value: PrimitiveType::Int64(5)},
            TC { input: "4 > 5;", left_value: PrimitiveType::Int64(4), operator: ">", right_value: PrimitiveType::Int64(5)},
            TC { input: "4 < 5;", left_value: PrimitiveType::Int64(4), operator: "<", right_value: PrimitiveType::Int64(5)},
            TC { input: "4 == 5;", left_value: PrimitiveType::Int64(4), operator: "==", right_value: PrimitiveType::Int64(5)},
            TC { input: "4 != 5;", left_value: PrimitiveType::Int64(4), operator: "!=", right_value: PrimitiveType::Int64(5)},
            TC { input: "true == true;", left_value:PrimitiveType::Bool(true), operator: "==", right_value: PrimitiveType::Bool(true)},
            TC { input: "true != false;", left_value:PrimitiveType::Bool(true), operator: "!=", right_value: PrimitiveType::Bool(false)},
            TC { input: "false == false;", left_value:PrimitiveType::Bool(false), operator: "==", right_value: PrimitiveType::Bool(false)},
        ];

        for tc in infix_tests.iter() {
            let lexer = lexer::Lexer::new(tc.input);
            let mut parser = parser::Parser::new(lexer);
            let program = parser.parse_program().expect("error parsing program");
            check_parser_errors(&parser);

            let statement = &program.statements[0];

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
            test_infix_expression(
                expression,
                tc.left_value.clone(),
                tc.operator.to_string(), 
                tc.right_value.clone()
                );
        }
    }

    #[test]
    fn test_operator_precendence_parsing() {
        struct TC<'a> {
            input: &'a str,
            expected: &'a str,
        }

        let tests = [
 
           TC {
                input: "!-a",
                expected: "(!(-a))",
            },

            TC {
                input: "-a * b",
                expected: "((-a) * b)",
            },
           TC {
                input: "a + b + c",
                expected: "((a + b) + c)",
            },
            TC {
                input: "a + b - c",
                expected: "((a + b) - c)",
            },
            TC {
                input: "a * b * c",
                expected: "((a * b) * c)",
            },
            TC {
                input: "a * b / c",
                expected: "((a * b) / c)",
            },
            TC {
                input: "a + b / c",
                expected: "(a + (b / c))",
            },
            TC {
                input: "a + b * c + d / e - f",
                expected: "(((a + (b * c)) + (d / e)) - f)",
            },
            TC {
                input: "3 + 4; -5 * 5",
                expected: "(3 + 4)((-5) * 5)",
            },
            TC {
                input: "5 > 4 == 3 < 4",
                expected: "((5 > 4) == (3 < 4))",
            },
            TC {
                input: "5 < 4 != 3 > 4",
                expected: "((5 < 4) != (3 > 4))",
            },
            TC {
                input: "3 + 4 * 5 == 3 * 1 + 4 * 5",
                expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            },
            TC {
                input: "true",
                expected: "true",
            },
            TC {
                input: "false",
                expected: "false",
            },
            TC {
                input: "3 > 5 == false",
                expected: "((3 > 5) == false)",
            },
            TC {
                input: "3 < 5 == true",
                expected: "((3 < 5) == true)",
            },
            TC {
                input: "1 + (2 + 3) + 4",
                expected: "((1 + (2 + 3)) + 4)",
            },
            TC{
                input: "(5 + 5) * 2",
                expected: "((5 + 5) * 2)",
            },
            TC{
                input: "2 / (5 + 5)",
                expected: "(2 / (5 + 5))",
            },
            TC{
                input: "-(5 + 5)",
                expected: "(-(5 + 5))",
            },
            TC{
                input: "!(true == true)",
                expected:"(!(true == true))",
            },
        ];

        for tc in tests.iter(){
            let lexer = lexer::Lexer::new(tc.input);
            let mut parser = parser::Parser::new(lexer);
            let program = parser.parse_program().expect("error parcing program");
            check_parser_errors(&parser);

            assert_eq!(
                program.string(), 
                tc.expected, 
                "expected=\"{}\", got=\"{}\"", 
                tc.expected,
                program.string()
                );
        }
        
    }

    #[test]
    fn test_boolean_expression(){
        let input = "true;";
        let lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program().expect("Error parsing program");
        check_parser_errors(&parser);

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

        let boolean_expression: &ast::BoxedExpression = match expression_statement.expression.as_ref() {
            Some(expr) => expr,
            None => panic!("no expression in ast::ExpressionStatement"),
        };

        test_literal_expression(boolean_expression, PrimitiveType::Bool(true));

    }

    #[test]
    fn test_if_expression(){
        let input :&str= "if (x < y) {x}";
        let lexer: lexer::Lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program().expect("error parsing program");
        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            1,
            "program.statements does not contain 1 statements, got={}",
            program.statements.len()
            );

        let expression_statement: &ast::ExpressionStatement = match program.statements[0] 
                .as_any()
                .downcast_ref::<ast::ExpressionStatement>(
            ) {
                Some(expr_stmt) => expr_stmt,
                None => panic!("program.statements[0] is not an ast::ExpressionStatement"),
            };

        let expression = match &expression_statement.expression {
            Some(exp) => exp,
            None => panic!("no expression")
        }; 

        let if_expression:&ast::IfExpression= match expression
            .as_any()
            .downcast_ref::<ast::IfExpression>(){
                Some(expr) => expr,
                None => panic!("expression_statement is not ast::IfExpression"),
        };

        test_infix_expression(
            &if_expression.condition, 
            PrimitiveType::Str("x".to_string()), 
            "<".to_string(), 
            PrimitiveType::Str("y".to_string())
            );

        assert_eq!(
            if_expression.consequence.statements.len(),
            1,
            "consequence is not 1 statements, got={}",
            if_expression.consequence.statements.len()
        );

        let consequence_expression_statement:&ast::ExpressionStatement= match &if_expression.consequence.statements[0] 
            .as_any()
            .downcast_ref::<ast::ExpressionStatement>(){
            Some(cons) => cons,
            None => panic!("if_expression.consequence.statemtns[0] is not an ast::ExpressionStatement")
        };

        let expression:&ast::BoxedExpression = match &consequence_expression_statement.expression {
            Some(exp) => exp,
            None => panic!("consequence_expression.expression is None")
        };

        test_identifier(&expression, "x".to_string());

        match &if_expression.alternative{
            Some(_)=>panic!("if_expression.alternative is not None"),
            None =>{}
        }
    }

    #[test]
    fn test_if_else_expression(){
        let input :&str= "if (x < y) {x} else {y}";
        let lexer: lexer::Lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let program = parser.parse_program().expect("error parsing program");
        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            1,
            "program.statements does not contain 1 statements, got={}",
            program.statements.len()
            );

        let expression_statement: &ast::ExpressionStatement = match program.statements[0] 
                .as_any()
                .downcast_ref::<ast::ExpressionStatement>(
            ) {
                Some(expr_stmt) => expr_stmt,
                None => panic!("program.statements[0] is not an ast::ExpressionStatement"),
            };

        let expression:&ast::BoxedExpression = match &expression_statement.expression{
            Some(exp) => exp,
            None=> panic!("expression_statement.expression is None") 
        };

        let if_expression:&ast::IfExpression =match expression
            .as_any()
            .downcast_ref::<ast::IfExpression>(){
                Some(expr) => expr,
                None => panic!("expression_statement is not ast::IfExpression"),
        };

        test_infix_expression(
            &if_expression.condition, 
            PrimitiveType::Str("x".to_string()), 
            "<".to_string(), 
            PrimitiveType::Str("y".to_string())
            );

        assert_eq!(
            if_expression.consequence.statements.len(),
            1,
            "consequence is not 1 statements, got={}",
            if_expression.consequence.statements.len()
        );

        let consequence_expression_statement:&ast::ExpressionStatement = match if_expression.consequence.statements[0] 
            .as_any()
            .downcast_ref::<ast::ExpressionStatement>(){
            Some(cons) => cons,
            None => panic!("if_expression.consequence.statemtns[0] is not an ast::ExpressionStatement")
        };

        let consequence_expression:&ast::BoxedExpression = match &consequence_expression_statement.expression {
            Some(exp) => exp,
            None => panic!("consequence_expression_statement.expression is None")
        };

        test_identifier(consequence_expression, "x".to_string());

        let alternative_expression_statement:&ast::BlockStatement = match &if_expression.alternative{
            Some(alter)=>alter,
            None =>panic!("if_expression.alternative is None")
        };

        assert_eq!(
            alternative_expression_statement.statements.len(),
            1,
            "consequence is not 1 statements, got={}",
            alternative_expression_statement.statements.len(),
        );

        let alternative_expression_statement: &ast::ExpressionStatement = match alternative_expression_statement.statements[0]
            .as_any()
            .downcast_ref::<ast::ExpressionStatement>(){
                Some(exp) => exp,
                None =>panic!("alternative_expression_statement.statement[0] is None")
        };

        let alternative_expression: &ast::BoxedExpression = match &alternative_expression_statement.expression{
            Some(exp) => exp,
            None=> panic!("alternative_expression_statement.expression is None")
        };

        test_identifier(alternative_expression, "y".to_string());
    }

}
