use crate::lexer::{token::Token, TokenizerRule};

pub(crate) struct KeywordRule {
    keyword: String,
    emits: Token,
}

impl KeywordRule {
    pub(crate) fn new(keyword: String, token: Token) -> Self {
        Self {
            keyword,
            emits: token,
        }
    }
}

impl TokenizerRule for KeywordRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String) {
        if fragment == self.keyword {
            return (Some(self.emits.clone()), String::new());
        }
        return (None, fragment);
    }
}

pub(crate) struct PatternRule {
    pattern: String,
    emits: Token,
}

impl PatternRule {
    pub(crate) fn new(pattern: String, emits: Token) -> Self {
        Self { pattern, emits }
    }
}

impl TokenizerRule for PatternRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String) {
        let l = self.pattern.len();

        if fragment.len() < l {
            return (None, fragment);
        }

        if fragment[0..l] == self.pattern {
            return (Some(self.emits.clone()), fragment[l..].to_string());
        }

        (None, fragment)
    }
}

pub(crate) struct StringLiteralRule;

impl TokenizerRule for StringLiteralRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String) {
        use super::token::LiteralToken::*;
        use Token::*;

        if fragment.starts_with("\"") && fragment.ends_with("\"") {
            return (
                Some(Literal(String(fragment[1..(fragment.len() - 1)].into()))),
                "".into(),
            );
        }

        (None, fragment)
    }
}

pub(crate) struct CharLiteralRule;

impl TokenizerRule for CharLiteralRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String) {
        use super::token::LiteralToken::*;
        use Token::*;

        if fragment.len() == 3 {
            let fragment: Vec<char> = fragment.chars().collect();
            if fragment[0] == '\'' && fragment[2] == '\'' {
                return (Some(Literal(Char(fragment[1].to_string()))), "".into());
            }
        }

        (None, fragment)
    }
}

pub(crate) struct NumberLiteralRule;

impl TokenizerRule for NumberLiteralRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String) {
        use super::token::LiteralToken::*;
        use Token::*;

        if fragment
            .chars()
            .next()
            .is_some_and(|c| c.is_numeric() || c == '-')
        {
            if fragment.contains('.') {
                return (Some(Literal(Decimal(fragment))), "".into());
            } else {
                return (Some(Literal(WholeNumber(fragment))), "".into());
            }
        }

        (None, fragment)
    }
}

pub(crate) struct BooleanLiteralRule;

impl TokenizerRule for BooleanLiteralRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String) {
        use super::token::LiteralToken::*;
        use Token::*;

        if fragment == "true" || fragment == "false" {
            return (Some(Literal(Boolean(fragment))), "".into());
        }

        (None, fragment)
    }
}

pub(crate) struct IdentifierRule;

impl TokenizerRule for IdentifierRule {
    fn try_apply(&self, fragment: String) -> (Option<Token>, String) {
        (Some(Token::Identifier(fragment)), String::new())
    }
}
