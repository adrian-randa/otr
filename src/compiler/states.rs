use crate::{compiler::{Compiler, CompilerEnvironment, CompilerError, CompilerState, states::{import::CompilerImportState, module::CompilerModuleState, r#struct::CompilerStructState}}, lexer::token::{KeywordToken, Token}, runtime::environment::{self, Environment}};

#[derive(Clone)]
pub struct CompilerBaseState {
    environment: Environment,
}

impl CompilerBaseState {
    pub fn new() -> Self {
        Self {
            environment: Environment::default(),
        }
    }
}

impl CompilerState for CompilerBaseState {
    fn read(self: Box<Self>, token: Token, _compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>, super::CompilerError> {
        match token {

            Token::Keyword(KeywordToken::Module) => {
                Ok(Box::new(CompilerModuleState::new(*self)))
            }

            Token::Keyword(KeywordToken::Import) => {
                Ok(Box::new(CompilerImportState::new(*self)))
            }

            _ => Err(CompilerError {
                message: format!("Unexpected token: {:?}", token)
            })
        }
    }

    fn finalize(self: Box<Self>) -> Result<Environment, super::CompilerError> {
        Ok(self.environment)
    }
}

pub mod module;
pub mod decorator;
pub mod procedure;
pub mod r#struct;
pub mod import;