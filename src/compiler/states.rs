use crate::{
    compiler::{
        CompilerEnvironment, CompilerError, CompilerState, states::{
            import::CompilerImportState, module::CompilerModuleState
        }
    }, core::CompiledObject, lexer::token::{KeywordToken, Token}
};

use crate::error::Result;

#[derive(Clone)]
pub struct CompilerBaseState {
    object: CompiledObject,
}

impl CompilerBaseState {
    pub fn new() -> Self {
        Self {
            object: CompiledObject::new(),
        }
    }
}

impl CompilerState for CompilerBaseState {
    fn read(
        self: Box<Self>,
        token: Token,
        _compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        match token {
            Token::Keyword(KeywordToken::Module) => {
                Ok(Box::new(CompilerModuleState::new(*self)) as Box<dyn CompilerState>)
            }

            Token::Keyword(KeywordToken::Import) => {
                Ok(Box::new(CompilerImportState::new(*self)) as Box<dyn CompilerState>)
            }

            other => Err(CompilerError::UnexpectedToken {
                expected: None,
                found: other,
            }
            .boxed()),
        }
    }

    fn finalize(self: Box<Self>) -> Result<CompiledObject> {
        Ok(self.object)
    }
}

pub mod decorator;
pub mod import;
pub mod module;
pub mod procedure;
pub mod r#struct;
