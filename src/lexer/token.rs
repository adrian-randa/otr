#[derive(Debug, Clone)]
pub(crate) enum Token {
    Keyword(KeywordToken),
    Operator(OperatorToken),
    Punctuation(PunctuationToken),
    Identifier(String),
    Literal(LiteralToken),
}

#[derive(Debug, Clone)]
pub(crate) enum KeywordToken {
    Let,
    Const,
    Proc,
    Struct,
    Return,
    For,
    While,
    Continue,
    Break,
    Module,
    Export,
}

#[derive(Debug, Clone)]
pub(crate) enum OperatorToken {
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
}

#[derive(Debug, Clone)]
pub(crate) enum ParenthesisType {
    Opening,
    Closing
}

#[derive(Debug, Clone)]
pub(crate) enum PunctuationToken {
    Parenthesis(ParenthesisType),
    SquareBrackets(ParenthesisType),
    CurlyBraces(ParenthesisType),
    Comma,
    Dot,
    DoubleColon,
    Semicolon,
    At,
}

#[derive(Debug, Clone)]
pub(crate) enum LiteralToken {
    WholeNumber(String),
    Decimal(String),
    Boolean(String),
    Char(String),
    String(String)
}

#[derive(Debug)]
pub struct TokenStream(pub(crate) Vec<Token>);