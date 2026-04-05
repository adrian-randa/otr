use crate::{
    compiler::{
        CompilerError, CompilerState, states::{CompilerBaseState, module::CompilerModuleState}
    }, core::module::{CompiledModule, ImportAddress}, lexer::token::{KeywordToken, LiteralToken, PunctuationToken, Token}
};

use crate::error::Result;

pub struct CompilerImportState {
    module_state: CompilerModuleState,
    module_address: Option<ImportAddress>,
}

impl CompilerState for CompilerImportState {
    fn read(
        mut self: Box<Self>,
        token: crate::lexer::token::Token,
        compiler_environment: &mut crate::compiler::CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        if self.module_address.is_none() {
            match token {
                Token::Identifier(ident) => {
                    self.module_address = Some(ImportAddress {
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
                    let address = self.module_address.unwrap();
                    compiler_environment.push_file_to_queue(address.clone());
                    self.module_state.get_module_mut().push_dependency(address);
                    return Ok(Box::new(self.module_state));
                }

                Token::Keyword(KeywordToken::From) => {
                    let module_id = self.module_address.as_mut().unwrap();

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
                    let module_id = self.module_address.as_mut().unwrap();
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

    fn finalize(self: Box<Self>) -> Result<CompiledModule> {
        Err(CompilerError::InvalidDefinition {
            message: "Unfinished module declaration!".into(),
        }
        .boxed())
    }
}

impl CompilerImportState {
    pub fn new(module_state: CompilerModuleState) -> Self {
        Self {
            module_state: module_state,
            module_address: None,
        }
    }
}
