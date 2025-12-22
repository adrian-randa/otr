use crate::{compiler::{Compiler, CompilerEnvironment, CompilerError, CompilerState, states::module::CompilerModuleState}, lexer::token::{KeywordToken, Token}, runtime::environment::{self, Environment}};

#[derive(Clone)]
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
    fn read(self: Box<Self>, token: Token, _compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>, super::CompilerError> {
        match token {

            Token::Keyword(KeywordToken::Module) => {
                Ok(Box::new(CompilerModuleState::new(*self)))
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