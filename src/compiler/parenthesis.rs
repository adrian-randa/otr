use crate::{error::{Result, compiler_error::CompilerError}, lexer::token::{ParenthesisType, PunctuationToken, Token}};

pub(crate) struct ParenthesisStack {
    stack: Vec<PunctuationToken>
}

impl ParenthesisStack {
    
    pub(crate) fn new() -> Self {
        Self {
            stack: Vec::new(),
        }
    }

    pub(crate) fn read(&mut self, token: Token) -> Result<()> {
        use PunctuationToken::*;
        use ParenthesisType::*;

        if let Token::Punctuation(punct) = token {
            match &punct {
                Parenthesis(p) |
                SquareBrackets(p) |
                CurlyBraces(p) => {
                    match p {
                        Opening => self.stack.push(punct),
                        Closing => {
                            let top = self.stack.pop().ok_or(CompilerError::InvalidParenthesisStructure.boxed())?;

                            match (&top, &punct) {
                                (Parenthesis(_), Parenthesis(_)) |
                                (SquareBrackets(_), SquareBrackets(_)) |
                                (CurlyBraces(_), CurlyBraces(_)) => {}
                                _ => {
                                    return Err(CompilerError::InvalidParenthesisStructure.boxed());
                                }                                        
                            }
                        },
                    }
                }

                _ => {}
            };
        }

        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.stack.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}