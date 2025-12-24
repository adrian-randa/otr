use crate::{compiler::{CompilerError, CompilerState, states::CompilerBaseState}, lexer::token::Token};

pub struct CompilerImportState {
    base_state: CompilerBaseState,
    module_id: Option<String>,
}

impl CompilerState for CompilerImportState {
    fn read(mut self: Box<Self>, token: crate::lexer::token::Token, compiler_environment: &mut crate::compiler::CompilerEnvironment) -> Result<Box<dyn CompilerState>, crate::compiler::CompilerError> {
        
        if self.module_id.is_none() {
            match token {
                Token::Identifier(ident) => {
                    self.module_id = Some(ident);
                    return Ok(self);
                }

                other => {
                    return Err(CompilerError {
                        message: format!("Unexpected token. Expected identifier, found {:?}!", other)
                    });
                }
            }
        } else {
            match token {
                Token::Punctuation(crate::lexer::token::PunctuationToken::Semicolon) => {
                    compiler_environment.get_file_reader_mut().enqueue(self.module_id.unwrap());
                    return Ok(Box::new(self.base_state))
                }
                
                other => {
                    return Err(CompilerError {
                        message: format!("Unexpected token. Expected ';', found {:?}!", other)
                    });
                }
            }
        }
    }

    fn finalize(self: Box<Self>) -> Result<crate::runtime::environment::Environment, crate::compiler::CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
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