#![allow(dead_code)]

pub type TokenType = String;

#[derive(Clone, Debug)]
pub struct Token {
    pub toke_type: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(token_t: &str, ch: &str) -> Self {
        Self {
            toke_type: token_t.to_string(),
            literal: ch.to_string(),
        }
    }
    pub fn new_empty() -> Self {
        Self {
            toke_type: "".to_string(),
            literal: "".to_string(),
        }
    }
}

pub const ILLEGAL: &str = "ILLEGAL";
pub const EOF: &str = "EOF";

// Identifiers + literals
pub const IDENT: &str = "IDENT";
pub const INT: &str = "INT";

// Operators
pub const ASSIGN: &str = "=";
pub const PLUS: &str = "+";
pub const MINUS: &str = "-";
pub const BANG: &str = "!";
pub const ASTERISK: &str = "*";
pub const SLASH: &str = "/";
pub const LT: &str = "<";
pub const GT: &str = ">";

pub const EQ: &str = "==";
pub const NOT_EQ: &str = "!=";
// Delimiters
pub const COMMA: &str = ",";
pub const SEMICOLON: &str = ";";
pub const LPAREN: &str = "(";
pub const RPAREN: &str = ")";
pub const LBRACE: &str = "{";
pub const RBRACE: &str = "}";

// Keywords
pub const FUNCTION: &str = "FUNCTION";
pub const LET: &str = "LET";
pub const TRUE: &str = "TRUE";
pub const FALSE: &str = "FALSE";
pub const IF: &str = "IF";
pub const ELSE: &str = "ELSE";
pub const RETURN: &str = "RETURN";

pub const KEYWORDS: [(&str, &str); 7] = [
    ("fn", FUNCTION),
    ("let", LET),
    ("true", TRUE),
    ("false", FALSE),
    ("if", IF),
    ("else", ELSE),
    ("return", RETURN),
];

pub fn lookup_identifier(ident: &str) -> &str {
    for kw in KEYWORDS {
        if ident == kw.0 {
            return kw.1;
        }
    }
    return IDENT;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_lookup_ident() {
        use crate::token::{lookup_identifier, FUNCTION, LET};
        assert_eq!(FUNCTION, lookup_identifier("fn"));
        assert_eq!(LET, lookup_identifier("let"));
    }
}
