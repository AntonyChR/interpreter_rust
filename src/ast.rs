#![allow(dead_code)]

use crate::token::Token;
use std::fmt::{self, write};

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'a> {
    Program(Program<'a>),
    Statement(Statement<'a>),
    Expression(Expression<'a>),
}

impl<'a> fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Statement(s) => write!(f, "{}", s),
            Node::Expression(e) => write!(f, "{}", e),
            Node::Program(p) => write!(f, "{}", p),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    Let(LetStatement<'a>),
    Return(ReturnStatement<'a>),
    Expression(ExpressionStatement<'a>),
    Block(BlockStatement<'a>),
}

impl<'a> fmt::Display for Statement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Let(s) => write!(f, "{}", s),
            Statement::Return(s) => write!(f, "{}", s),
            Statement::Expression(s) => write!(f, "{}", s),
            Statement::Block(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    Identifier(Identifier<'a>),
    IntegerLiteral(IntegerLiteral<'a>),
    Prefix(PrefixExpression<'a>),
    Infix(InfixExpression<'a>),
    Boolean(Boolean<'a>),
    If(IfExpression<'a>),
    FunctionLiteral(FunctionLiteral<'a>),
    Call(CallExpression<'a>),
    StringLiteral(StringLiteral<'a>)
}

impl<'a> fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Identifier(e) => write!(f, "{}", e),
            Expression::IntegerLiteral(e) => write!(f, "{}", e),
            Expression::Prefix(e) => write!(f, "{}", e),
            Expression::Infix(e) => write!(f, "{}", e),
            Expression::Boolean(e) => write!(f, "{}", e),
            Expression::If(e) => write!(f, "{}", e),
            Expression::FunctionLiteral(e) => write!(f, "{}", e),
            Expression::Call(e) => write!(f, "{}", e),
            Expression::StringLiteral(e) => write!(f, "{}", e),
        }
    }
}

// --- Root Node ---

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl<'a> fmt::Display for Program<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl<'a> Program<'a> {
    pub fn string(&self) -> String {
        self.to_string()
    }
}

// --- Concrete Structs ---

#[derive(Debug, Clone, PartialEq)]
pub struct LetStatement<'a> {
    pub token: Token<'a>, // the 'let' token
    pub name: Identifier<'a>,
    pub value: Expression<'a>,
}

impl<'a> fmt::Display for LetStatement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} = {};", self.token.literal, self.name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement<'a> {
    pub token: Token<'a>, // the 'return' token
    pub return_value: Option<Box<Expression<'a>>>,
}

impl<'a> fmt::Display for ReturnStatement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.token.literal)?;
        if let Some(val) = &self.return_value {
            write!(f, "{}", val)?;
        }
        write!(f, ";")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStatement<'a> {
    pub token: Token<'a>, // the first token of the expression
    pub expression: Box<Expression<'a>>,
}

impl<'a> fmt::Display for ExpressionStatement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expression)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement<'a> {
    pub token: Token<'a>, // the '{' token
    pub statements: Vec<Statement<'a>>,
}

impl<'a> fmt::Display for BlockStatement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    pub token: Token<'a>,
    pub value: &'a str,
}

impl<'a> fmt::Display for Identifier<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerLiteral<'a> {
    pub token: Token<'a>,
    pub value: i64,
}

impl<'a> fmt::Display for IntegerLiteral<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixExpression<'a> {
    pub token: Token<'a>, // e.g., '!', '-'
    pub operator: &'a str,
    pub right: Box<Expression<'a>>,
}

impl<'a> fmt::Display for PrefixExpression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}{})", self.operator, self.right)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfixExpression<'a> {
    pub token: Token<'a>, // e.g., '+'
    pub left: Box<Expression<'a>>,
    pub operator: &'a str,
    pub right: Box<Expression<'a>>,
}

impl<'a> fmt::Display for InfixExpression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {} {})", self.left, self.operator, self.right)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean<'a> {
    pub token: Token<'a>,
    pub value: bool,
}

impl<'a> fmt::Display for Boolean<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token.literal)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpression<'a> {
    pub token: Token<'a>, // The 'if' token
    pub condition: Box<Expression<'a>>,
    pub consequence: BlockStatement<'a>,
    pub alternative: Option<BlockStatement<'a>>,
}

impl<'a> fmt::Display for IfExpression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if {} {}", self.condition, self.consequence)?;
        if let Some(alt) = &self.alternative {
            write!(f, " else {}", alt)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionLiteral<'a> {
    pub token: Token<'a>, // The 'fn' token
    pub parameters: Vec<Identifier<'a>>,
    pub body: BlockStatement<'a>,
}

impl<'a> fmt::Display for FunctionLiteral<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params: Vec<String> = self.parameters.iter().map(|p| p.to_string()).collect();
        write!(
            f,
            "{}({}) {}",
            self.token.literal,
            params.join(", "),
            self.body
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression<'a> {
    pub token: Token<'a>,              // The '(' token
    pub function: Box<Expression<'a>>, // Identifier or FunctionLiteral
    pub arguments: Vec<Expression<'a>>,
}

impl<'a> fmt::Display for CallExpression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<String> = self.arguments.iter().map(|a| a.to_string()).collect();
        write!(f, "{}({})", self.function, args.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral<'a> {
    pub token: Token<'a>,
    pub value: String,
}

impl<'a> fmt::Display for StringLiteral<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f,r#""{}""#, self.value) 
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_display() {
        let program = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token {
                    toke_type: TokenType::Let,
                    literal: "let",
                },
                name: Identifier {
                    token: Token {
                        toke_type: TokenType::Ident,
                        literal: "myVar",
                    },
                    value: "myVar",
                },
                value: Expression::Identifier(Identifier {
                    token: Token {
                        toke_type: TokenType::Ident,
                        literal: "anotherVar",
                    },
                    value: "anotherVar",
                }),
            })],
        };

        assert_eq!(program.to_string(), "let myVar = anotherVar;");
    }
}
