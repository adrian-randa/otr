use crate::{compiler::{CompilerError, CompilerState, states::module::CompilerModuleState}, lexer::token::{KeywordToken, Token}, runtime::environment::{self, Environment}};

pub struct CompilerBaseState {
    environment: Environment,
}

impl CompilerBaseState {
    pub fn new() -> Self {
        Self {
            environment: Environment::new("".into()),
        }
    }
}

impl CompilerState for CompilerBaseState {
    fn read(self, token: Token) -> Result<Box<dyn CompilerState>, super::CompilerError> {
        match token {

            Token::Keyword(KeywordToken::Module) => {
                Ok(Box::new(CompilerModuleState::new(self)))
            }

            _ => Err(CompilerError {
                message: format!("Unexpected token: {:?}", token)
            })
        }
    }

    fn finalize(self) -> Result<Environment, super::CompilerError> {
        Ok(self.environment)
    }
}

pub mod module;
pub mod decorator;
pub mod procedure;