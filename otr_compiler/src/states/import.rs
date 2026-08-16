use crate::{
    CompilerEnvironment, CompilerError, CompilerState, lexer::token::{KeywordToken, LiteralToken, PunctuationToken, Token}, states::{module::CompilerModuleState}
};

use otr_core::{module::{CompiledModule, ImportAddress}, error::Result};

pub struct CompilerImportState {
    module: Box<CompilerModuleState>,
    import_address: Option<ImportAddress>,
}

impl CompilerState for CompilerImportState {
    fn read(
        mut self: Box<Self>,
        token: Token,
        compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        if let Some(ref mut address) = self.import_address {
            match token {
                Token::Punctuation(PunctuationToken::Semicolon) => {
                    compiler_environment.push_file_to_queue(address.clone());
                    self.module.get_module_mut().push_dependency(address.clone());
                    Ok(self.module)
                }

                Token::Keyword(KeywordToken::From) => {
                    if address.path.is_some() {
                        return Err(CompilerError::InvalidDefinition {
                            message: "Cannot declare more than one location for an import!".into(),
                        }
                        .boxed());
                    }

                    address.path = Some(String::new());

                    Ok(self)
                }

                Token::Literal(LiteralToken::String(path)) => {
                    if address.path.is_some() {
                        address.path = Some(path);
                        Ok(self)
                    } else {
                        Err(CompilerError::InvalidDefinition {
                            message: "Unexpected String literal. Try adding 'from' to declare a location for an import!".into()
                        }.boxed())
                    }
                }

                Token::Keyword(KeywordToken::As) => {
                    if address.alias.is_some() {
                        return Err(CompilerError::InvalidDefinition {
                            message: "Cannot declare more than one alias for an import!".into()
                        }.boxed());
                    }

                    address.alias = Some(String::new());

                    Ok(self)
                }

                Token::Identifier(alias) => {
                    if address.alias.is_some() {
                        address.alias = Some(alias);
                        Ok(self)
                    } else {
                        Err(CompilerError::InvalidDefinition {
                            message: "Unexpected Identifier. Try adding 'as' to declare an alias for an import!".into()
                        }.boxed())
                    }
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some(";".into()),
                        found: other,
                    }
                    .boxed())
                }
            }
        } else {
            match token {
                Token::Identifier(ident) => {
                    self.import_address = Some(ImportAddress {
                        module_id: ident,
                        path: None,
                        alias: None,
                    });
                    Ok(self)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("Identifier".into()),
                        found: other,
                    }
                    .boxed())
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
    pub fn new(module: Box<CompilerModuleState>) -> Self {
        Self {
            module,
            import_address: None,
        }
    }
}
