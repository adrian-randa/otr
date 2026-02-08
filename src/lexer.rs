use std::str::{Chars, FromStr};

use derive_more::IntoIterator;

use crate::error::{Error, Result};

use crate::lexer::token::{ContextualizedToken, ContextualizedTokenStream};
use crate::{error::{fragmenter_error::FragmentationError, tokenizer_error::TokenizerError}, lexer::{
    rules::{
        BooleanLiteralRule, CharLiteralRule, IdentifierRule, KeywordRule, NumberLiteralRule,
        PatternRule, StringLiteralRule,
    },
    token::{Token, TokenStream},
}};

pub mod rules;
pub mod token;

struct CharCoordinateIterator<'a> {
    iter: Chars<'a>,
    line: usize,
    column: usize,
}

impl<'a> Iterator for CharCoordinateIterator<'a> {
    type Item = (char, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.iter.next()?;
        self.column += 1;
        if c == '\n' {
            self.column = 1;
            self.line += 1;
        }

        Some((c, self.line, self.column))
    }
}

#[derive(Debug)]
pub struct Fragment {
    fragment: String,
    line_index: usize,
    column_index: usize,
}

#[derive(Debug, IntoIterator)]
pub struct FragmentStream(Vec<Fragment>);

impl FromStr for FragmentStream {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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
        let mut current_pos = (0, 0);
        let mut current_kind = CharKind::Alphabetic;

        let s = s.to_string();

        let chars: Vec<(char, usize, usize)> = CharCoordinateIterator {
            iter: s.chars(),
            line: 1,
            column: 1,
        }.collect();

        let (_, line, column) = chars.last().unwrap();

        let mut i = 0;

        while i < chars.len() {
            let (c, line, column) = chars[i];

            i += 1;

            if c == '\'' {
                if !current.is_empty() {
                    stream.push(
                        Fragment {
                            fragment: current,
                            line_index: current_pos.0,
                            column_index: current_pos.1,
                        }
                    );
                    current_pos = (line, column);
                    current = String::new();
                }

                current.push('\'');

                current.push(chars[i].0);

                current.push('\'');

                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                current_pos = (line, column);
                current = String::new();

                i += 2;
                continue;
            }

            if c == '\"' {
                if !current.is_empty() {
                    stream.push(Fragment {
                        fragment: current,
                        line_index: current_pos.0,
                        column_index: current_pos.1,
                    });
                    current_pos = (line, column);
                    current = String::new();
                }

                current.push('\"');

                while chars[i].0 != '\"' {
                    if chars[i].0 == '\\' {
                        match chars[i + 1].0 {
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
                            _ => return Err(FragmentationError::InvalidControlCharacter {
                                line_index: line,
                                column_index: column
                            }.boxed()),
                        }
                        i = i + 2;
                        continue;
                    }
                    if chars[i].0 == '\n' {
                        return Err(FragmentationError::LinebreakInStringLiteral { line_index: line, column_index: column }.boxed())
                    }

                    current.push(chars[i].0);

                    i += 1;
                    
                }

                current.push('\"');

                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                current_pos = (line, column);
                current = String::new();

                i += 1;
                continue;
            }

            if c.is_ascii_whitespace() {
                if current.is_empty() {
                    current_pos = (line, column);
                    continue;
                }
                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                current_pos = (line, column);
                current = String::new();
                continue;
            }

            if c == '#' {
                if !current.is_empty() {
                    stream.push(Fragment {
                        fragment: current,
                        line_index: current_pos.0,
                        column_index: current_pos.1,
                    });
                    current_pos = (line, column);
                    current = String::new();
                }

                while chars[i].0 != '\n' && i < chars.len() {
                    i += 1;
                }

                continue;
            }

            if c == ';' {
                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                stream.push(Fragment {
                    fragment: ";".into(),
                    line_index: current_pos.0,
                    column_index: column,
                });
                current_pos = (line, column);
                current = String::new();
                continue;
            }

            let next_char_kind: CharKind = c.into();

            if !current.is_empty() {
                use CharKind::*;
                match (current_kind, next_char_kind) {
                    (Alphabetic, Punctuation)
                    | (Punctuation, Alphabetic)
                    /*| (Numeric, Alphabetic) */ => {
                        stream.push(Fragment {
                            fragment: current,
                            line_index: current_pos.0,
                            column_index: current_pos.1,
                        });
                        current_pos = (line, column);
                        current = String::new();
                    }
                    (Numeric, Punctuation) => {
                        if c != '.' {
                            stream.push(Fragment {
                                fragment: current,
                                line_index: current_pos.0,
                                column_index: current_pos.1,
                            });
                            current_pos = (line, column);
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
            stream.push(Fragment {
                fragment: current,
                line_index: current_pos.0,
                column_index: current_pos.1,
            });
        }

        Ok(Self(stream))
    }
}

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
        use token::*;
        use KeywordToken::*;
        use OperatorToken::*;
        use ParenthesisType::*;
        use PunctuationToken::*;
        use Token::*;
        use LiteralToken::*;

        Self::new()
            .with_rule(KeywordRule::new("break".into(), Keyword(Break)))
            .with_rule(KeywordRule::new("const".into(), Keyword(Const)))
            .with_rule(KeywordRule::new("continue".into(), Keyword(Continue)))
            .with_rule(KeywordRule::new("for".into(), Keyword(For)))
            .with_rule(KeywordRule::new("in".into(), Keyword(In)))
            .with_rule(KeywordRule::new("let".into(), Keyword(Let)))
            .with_rule(KeywordRule::new("proc".into(), Keyword(Proc)))
            .with_rule(KeywordRule::new("return".into(), Keyword(Return)))
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

            .with_rule(KeywordRule::new("Null".into(), Literal(LiteralToken::Null)))
            .with_rule(KeywordRule::new("Integer".into(), Literal(Type(PrimitiveTypeToken::Integer))))
            .with_rule(KeywordRule::new("Float".into(), Literal(Type(PrimitiveTypeToken::Float))))
            .with_rule(KeywordRule::new("Bool".into(), Literal(Type(PrimitiveTypeToken::Bool))))
            .with_rule(KeywordRule::new("Char".into(), Literal(Type(PrimitiveTypeToken::Char))))
            .with_rule(KeywordRule::new("String".into(), Literal(Type(PrimitiveTypeToken::String))))
            .with_rule(KeywordRule::new("Array".into(), Literal(Type(PrimitiveTypeToken::Array))))
            .with_rule(KeywordRule::new("Type".into(), Literal(Type(PrimitiveTypeToken::Type))))
            .with_rule(KeywordRule::new("Moved".into(), Literal(Type(PrimitiveTypeToken::Moved))))
            .with_rule(KeywordRule::new("Dropeed".into(), Literal(Type(PrimitiveTypeToken::Dropped))))

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
