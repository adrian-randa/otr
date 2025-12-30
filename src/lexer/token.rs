use derive_more::IntoIterator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Keyword(KeywordToken),
    Operator(OperatorToken),
    Punctuation(PunctuationToken),
    Identifier(String),
    Literal(LiteralToken),
    PrimitiveType(PrimitiveTypeToken)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeywordToken {
    Let,
    Const,
    Proc,
    Struct,
    Return,
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
    Ref,
    Clone,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralToken {
    Null,
    Integer(String),
    Decimal(String),
    Boolean(String),
    Char(String),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveTypeToken {
    Integer,
    Decimal,
    Boolean,
    Char,
    String,
    Array,
}

#[derive(Debug, IntoIterator)]
pub struct TokenStream(pub Vec<Token>);
