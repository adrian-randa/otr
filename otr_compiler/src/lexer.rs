use otr_core::error::Result;

use crate::lexer::{
    fragmenter::{Fragment, FragmentStream},
    rules::{
        BooleanLiteralRule, CharLiteralRule, IdentifierRule, KeywordRule, NumberLiteralRule,
        PatternRule, StringLiteralRule,
    },
    token::{ContextualizedToken, ContextualizedTokenStream, Token},
};

pub(crate) mod error;
pub mod fragmenter;
pub mod rules;
pub mod token;

pub trait TokenizerRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String);
}

pub struct Tokenizer {
    rules: Vec<Box<dyn TokenizerRule>>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    pub fn with_rule(mut self, rule: impl TokenizerRule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn tokenize(&self, fragments: FragmentStream) -> Result<ContextualizedTokenStream> {
        let mut stream = Vec::new();

        for mut frag in fragments {
            'scan: while !frag.fragment.is_empty() {
                for rule in self.rules.iter() {
                    let frag_len = frag.fragment.len();
                    let (token, remainder) = rule.try_apply(frag.fragment);
                    let rem_len = remainder.len();
                    let line = frag.line_index;
                    let column = frag.column_index;
                    frag = Fragment {
                        fragment: remainder,
                        line_index: frag.line_index,
                        column_index: frag.column_index + frag_len - rem_len,
                    };

                    if let Some(token) = token {
                        stream.push(ContextualizedToken {
                            token,
                            line_index: line,
                            column_index: column,
                        });
                        continue 'scan;
                    }
                }
            }
        }

        Ok(ContextualizedTokenStream(stream))
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        use KeywordToken::*;
        use LiteralToken::*;
        use OperatorToken::*;
        use ParenthesisType::*;
        use PunctuationToken::*;
        use Token::*;
        use token::*;

        Self::new()
            .with_rule(KeywordRule::new("break".into(), Keyword(Break)))
            .with_rule(KeywordRule::new("const".into(), Keyword(Const)))
            .with_rule(KeywordRule::new("continue".into(), Keyword(Continue)))
            .with_rule(KeywordRule::new("for".into(), Keyword(For)))
            .with_rule(KeywordRule::new("in".into(), Keyword(In)))
            .with_rule(KeywordRule::new("let".into(), Keyword(Let)))
            .with_rule(KeywordRule::new("proc".into(), Keyword(Proc)))
            .with_rule(KeywordRule::new("return".into(), Keyword(Return)))
            .with_rule(KeywordRule::new("throw".into(), Keyword(Throw)))
            .with_rule(KeywordRule::new("catch".into(), Keyword(Catch)))
            .with_rule(KeywordRule::new("struct".into(), Keyword(Struct)))
            .with_rule(KeywordRule::new("while".into(), Keyword(While)))
            .with_rule(KeywordRule::new("if".into(), Keyword(If)))
            .with_rule(KeywordRule::new("else".into(), Keyword(Else)))
            .with_rule(KeywordRule::new("module".into(), Keyword(Module)))
            .with_rule(KeywordRule::new("export".into(), Keyword(Export)))
            .with_rule(KeywordRule::new("import".into(), Keyword(Import)))
            .with_rule(KeywordRule::new("from".into(), Keyword(From)))
            .with_rule(KeywordRule::new("public".into(), Keyword(Public)))
            .with_rule(KeywordRule::new("ref".into(), Keyword(Ref)))
            .with_rule(KeywordRule::new("clone".into(), Keyword(Clone)))
            .with_rule(KeywordRule::new("typeof".into(), Keyword(Typeof)))
            .with_rule(KeywordRule::new("null".into(), Literal(LiteralToken::Null)))
            .with_rule(KeywordRule::new(
                "Null".into(),
                Literal(Type(PrimitiveTypeToken::Null)),
            ))
            .with_rule(KeywordRule::new(
                "Integer".into(),
                Literal(Type(PrimitiveTypeToken::Integer)),
            ))
            .with_rule(KeywordRule::new(
                "Float".into(),
                Literal(Type(PrimitiveTypeToken::Float)),
            ))
            .with_rule(KeywordRule::new(
                "Bool".into(),
                Literal(Type(PrimitiveTypeToken::Bool)),
            ))
            .with_rule(KeywordRule::new(
                "Char".into(),
                Literal(Type(PrimitiveTypeToken::Char)),
            ))
            .with_rule(KeywordRule::new(
                "String".into(),
                Literal(Type(PrimitiveTypeToken::String)),
            ))
            .with_rule(KeywordRule::new(
                "Array".into(),
                Literal(Type(PrimitiveTypeToken::Array)),
            ))
            .with_rule(KeywordRule::new(
                "Type".into(),
                Literal(Type(PrimitiveTypeToken::Type)),
            ))
            .with_rule(KeywordRule::new(
                "Moved".into(),
                Literal(Type(PrimitiveTypeToken::Moved)),
            ))
            .with_rule(KeywordRule::new(
                "Dropeed".into(),
                Literal(Type(PrimitiveTypeToken::Dropped)),
            ))
            .with_rule(PatternRule::new("&&".into(), Operator(And)))
            .with_rule(PatternRule::new("||".into(), Operator(Or)))
            .with_rule(PatternRule::new("==".into(), Operator(Equality)))
            .with_rule(PatternRule::new("!=".into(), Operator(Inequality)))
            .with_rule(PatternRule::new("::".into(), Punctuation(DoubleColon)))
            .with_rule(PatternRule::new(">=".into(), Operator(GreaterEquals)))
            .with_rule(PatternRule::new("<=".into(), Operator(LessEquals)))
            .with_rule(PatternRule::new("->".into(), Punctuation(ThinArrow)))
            .with_rule(PatternRule::new(">".into(), Operator(Greater)))
            .with_rule(PatternRule::new("<".into(), Operator(Less)))
            .with_rule(PatternRule::new(
                "(".into(),
                Punctuation(Parenthesis(Opening)),
            ))
            .with_rule(PatternRule::new(
                ")".into(),
                Punctuation(Parenthesis(Closing)),
            ))
            .with_rule(PatternRule::new(
                "[".into(),
                Punctuation(SquareBrackets(Opening)),
            ))
            .with_rule(PatternRule::new(
                "]".into(),
                Punctuation(SquareBrackets(Closing)),
            ))
            .with_rule(PatternRule::new(
                "{".into(),
                Punctuation(CurlyBraces(Opening)),
            ))
            .with_rule(PatternRule::new(
                "}".into(),
                Punctuation(CurlyBraces(Closing)),
            ))
            .with_rule(NumberLiteralRule)
            .with_rule(PatternRule::new("@".into(), Punctuation(At)))
            .with_rule(PatternRule::new("!".into(), Operator(Not)))
            .with_rule(PatternRule::new("+".into(), Operator(Plus)))
            .with_rule(PatternRule::new("-".into(), Operator(Minus)))
            .with_rule(PatternRule::new("*".into(), Operator(Multiply)))
            .with_rule(PatternRule::new("/".into(), Operator(Divide)))
            .with_rule(PatternRule::new("%".into(), Operator(Modulo)))
            .with_rule(PatternRule::new("=".into(), Operator(Assignment)))
            .with_rule(PatternRule::new("^".into(), Operator(Power)))
            .with_rule(PatternRule::new(",".into(), Punctuation(Comma)))
            .with_rule(PatternRule::new(".".into(), Punctuation(Dot)))
            .with_rule(PatternRule::new(":".into(), Punctuation(Colon)))
            .with_rule(PatternRule::new(";".into(), Punctuation(Semicolon)))
            .with_rule(StringLiteralRule)
            .with_rule(CharLiteralRule)
            .with_rule(BooleanLiteralRule)
            .with_rule(IdentifierRule)
    }
}
