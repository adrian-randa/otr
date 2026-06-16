use std::ops::Deref;

use derive_more::{Deref, IntoIterator};

#[derive(Debug)]
pub struct ContextualizedToken {
    pub token: Token,
    pub line_index: usize,
}

impl Deref for ContextualizedToken {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Keyword(KeywordToken),
    Operator(OperatorToken),
    Punctuation(PunctuationToken),
    Identifier(String),
    Literal(LiteralToken),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeywordToken {
    Let,
    Const,
    Proc,
    Struct,
    Return,
    Throw,
    Catch,
    For,
    While,
    If,
    Else,
    Continue,
    Break,
    Module,
    Export,
    Import,
    From,
    Public,
    Is,
    In,
    Ref,
    Clone,
    Typeof,
    Using,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OperatorToken {
    Assignment,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Power,
    Not,
    And,
    Or,
    Equality,
    Inequality,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParenthesisType {
    Opening,
    Closing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PunctuationToken {
    Parenthesis(ParenthesisType),
    SquareBrackets(ParenthesisType),
    CurlyBraces(ParenthesisType),
    Comma,
    Dot,
    Colon,
    DoubleColon,
    Semicolon,
    At,
    ThinArrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralToken {
    Null,
    Integer(String),
    Float(String),
    Boolean(String),
    Char(String),
    String(String),
    Type(PrimitiveTypeToken),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveTypeToken {
    Null,
    Integer,
    Float,
    Bool,
    Char,
    String,
    Array,
    Moved,
    Dropped,
    Type,
}

#[derive(Debug, IntoIterator)]
pub struct TokenStream(pub Vec<Token>);

#[derive(Debug, IntoIterator)]
pub struct ContextualizedTokenStream(pub Vec<ContextualizedToken>);