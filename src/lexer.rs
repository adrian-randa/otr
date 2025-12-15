use std::str::FromStr;

use derive_more::IntoIterator;

use crate::lexer::{
    rules::{
        BooleanLiteralRule, CharLiteralRule, IdentifierRule, KeywordRule, NumberLiteralRule,
        PatternRule, StringLiteralRule,
    },
    token::{Token, TokenStream},
};

pub mod rules;
pub mod token;

#[derive(Debug, IntoIterator)]
pub struct FragmentStream(Vec<String>);

#[derive(Debug)]
pub enum FragmentationError {
    InvalidControlCharacter,
}

impl FromStr for FragmentStream {
    type Err = FragmentationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stream = Vec::new();

        #[derive(Debug, PartialEq)]
        enum CharKind {
            Alphabetic,
            Numeric,
            Punctuation,
        }

        impl From<char> for CharKind {
            fn from(value: char) -> Self {
                if value.is_ascii_alphabetic() {
                    return Self::Alphabetic;
                }
                if value.is_numeric() {
                    return Self::Numeric;
                }
                if value.is_ascii_punctuation() {
                    return Self::Punctuation;
                }

                panic!("Unsupported char kind");
            }
        }

        let mut current = String::new();
        let mut current_kind = CharKind::Alphabetic;

        let chars: Vec<char> = s.chars().collect();

        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            i += 1;

            if c == '\'' {
                if !current.is_empty() {
                    stream.push(current);
                    current = String::new();
                }

                current.push('\'');

                current.push(chars[i]);

                current.push('\'');

                i += 2;
                continue;
            }

            if c == '\"' {
                if !current.is_empty() {
                    stream.push(current);
                    current = String::new();
                }

                current.push('\"');

                while chars[i] != '\"' {
                    if chars[i] == '\\' {
                        match chars[i + 1] {
                            'n' => {
                                current.push('\n');
                            }
                            't' => {
                                current.push('\t');
                            }
                            '\"' => {
                                current.push('\"');
                            }
                            '\\' => {
                                current.push('\\');
                            }
                            _ => return Err(FragmentationError::InvalidControlCharacter),
                        }
                        i = i + 2;
                        continue;
                    }

                    current.push(chars[i]);

                    i += 1;
                }

                current.push('\"');

                stream.push(current);
                current = String::new();

                i += 1;
                continue;
            }

            if c.is_ascii_whitespace() {
                if current.is_empty() {
                    continue;
                }
                stream.push(current);
                current = String::new();
                continue;
            }

            if c == '#' {
                if !current.is_empty() {
                    stream.push(current);
                    current = String::new();
                }

                while chars[i] != '\n' && i < chars.len() {
                    i += 1;
                }

                continue;
            }

            if c == ';' {
                stream.push(current);
                stream.push(";".into());
                current = String::new();
                continue;
            }

            let next_char_kind: CharKind = c.into();

            if !current.is_empty() {
                use CharKind::*;
                match (current_kind, next_char_kind) {
                    (Alphabetic, Punctuation)
                    | (Punctuation, Alphabetic)
                    | (Numeric, Alphabetic) => {
                        stream.push(current);
                        current = String::new();
                    }
                    (Numeric, Punctuation) => {
                        if c != '.' {
                            stream.push(current);
                            current = String::new();
                        }
                    }

                    _ => {}
                }
            }

            current_kind = c.into();

            current.push(c);
        }

        if !current.is_empty() {
            stream.push(current);
        }

        Ok(Self(stream))
    }
}

#[derive(Debug)]
pub enum TokenizeError {}

trait TokenizerRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String);
}

pub struct Tokenizer {
    rules: Vec<Box<dyn TokenizerRule>>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    fn with_rule(mut self, rule: impl TokenizerRule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn tokenize(&self, fragments: FragmentStream) -> Result<TokenStream, TokenizeError> {
        let mut stream = Vec::new();

        for mut frag in fragments {
            'scan: while !frag.is_empty() {
                for rule in self.rules.iter() {
                    let token;
                    (token, frag) = rule.try_apply(frag);

                    if let Some(token) = token {
                        stream.push(token);
                        continue 'scan;
                    }
                }
            }
        }

        Ok(TokenStream(stream))
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        use token::*;
        use KeywordToken::*;
        use OperatorToken::*;
        use ParenthesisType::*;
        use PunctuationToken::*;
        use Token::*;

        Self::new()
            .with_rule(KeywordRule::new("break".into(), Keyword(Break)))
            .with_rule(KeywordRule::new("const".into(), Keyword(Const)))
            .with_rule(KeywordRule::new("continue".into(), Keyword(Continue)))
            .with_rule(KeywordRule::new("for".into(), Keyword(For)))
            .with_rule(KeywordRule::new("let".into(), Keyword(Let)))
            .with_rule(KeywordRule::new("proc".into(), Keyword(Proc)))
            .with_rule(KeywordRule::new("return".into(), Keyword(Return)))
            .with_rule(KeywordRule::new("struct".into(), Keyword(Struct)))
            .with_rule(KeywordRule::new("while".into(), Keyword(While)))
            .with_rule(KeywordRule::new("module".into(), Keyword(Module)))
            .with_rule(KeywordRule::new("export".into(), Keyword(Export)))
            .with_rule(PatternRule::new("&&".into(), Operator(And)))
            .with_rule(PatternRule::new("||".into(), Operator(Or)))
            .with_rule(PatternRule::new("==".into(), Operator(Equality)))
            .with_rule(PatternRule::new("!=".into(), Operator(Inequality)))
            .with_rule(PatternRule::new("::".into(), Punctuation(DoubleColon)))
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
            .with_rule(NumberLiteralRule)
            .with_rule(StringLiteralRule)
            .with_rule(CharLiteralRule)
            .with_rule(BooleanLiteralRule)
            .with_rule(IdentifierRule)
    }
}
