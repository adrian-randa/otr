use crate::{
    compiler::{
        CompilerError, CompilerState, source_file_reader::ImportAddress, states::CompilerBaseState
    }, core::CompiledObject, lexer::token::{KeywordToken, LiteralToken, PunctuationToken, Token}
};

use crate::error::Result;

pub struct CompilerImportState {
    base_state: CompilerBaseState,
    module_id: Option<ImportAddress>,
}

impl CompilerState for CompilerImportState {
    fn read(
        mut self: Box<Self>,
        token: crate::lexer::token::Token,
        compiler_environment: &mut crate::compiler::CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        if self.module_id.is_none() {
            match token {
                Token::Identifier(ident) => {
                    self.module_id = Some(ImportAddress {
                        module_id: ident,
                        path: None,
                    });
                    return Ok(self);
                }

                other => {
                    return Err(CompilerError::UnexpectedToken {
                        expected: Some("Identifier".into()),
                        found: other,
                    }
                    .boxed());
                }
            }
        } else {
            match token {
                Token::Punctuation(PunctuationToken::Semicolon) => {
                    compiler_environment
                        .get_file_reader_mut()
                        .push_dependency(self.module_id.unwrap())?;
                    return Ok(Box::new(self.base_state));
                }

                Token::Keyword(KeywordToken::From) => {
                    let module_id = self.module_id.as_mut().unwrap();

                    if module_id.path.is_some() {
                        return Err(CompilerError::InvalidDefinition {
                            message: "Cannot declare more than one location for an import!".into(),
                        }
                        .boxed());
                    }

                    module_id.path = Some(String::new());

                    return Ok(self);
                }

                Token::Literal(LiteralToken::String(path)) => {
                    let module_id = self.module_id.as_mut().unwrap();
                    if module_id.path.is_some() {
                        module_id.path = Some(path);
                        return Ok(self);
                    } else {
                        return Err(CompilerError::InvalidDefinition {
                            message: "Unexpected String literal. Try adding 'from' to declare a location for an import!".into()
                        }.boxed());
                    }
                }

                other => {
                    return Err(CompilerError::UnexpectedToken {
                        expected: Some(";".into()),
                        found: other,
                    }
                    .boxed());
                }
            }
        }
    }

    fn finalize(self: Box<Self>) -> Result<CompiledObject> {
        Err(CompilerError::InvalidDefinition {
            message: "Unfinished module declaration!".into(),
        }
        .boxed())
    }
}

impl CompilerImportState {
    pub fn new(base_state: CompilerBaseState) -> Self {
        Self {
            base_state,
            module_id: None,
        }
    }
}
