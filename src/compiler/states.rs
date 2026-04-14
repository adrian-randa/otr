use crate::{
    compiler::{
        lexer::token::{KeywordToken, Token},
        CompilerEnvironment, CompilerError, CompilerState, states::module::CompilerModuleState
    }, core::{module::CompiledModule}
};

use crate::error::Result;

#[derive(Clone)]
pub(crate) struct CompilerBaseState {
    module: Option<CompiledModule>,
}

impl CompilerBaseState {
    pub fn new() -> Self {
        Self {
            module: None,
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

            other => Err(CompilerError::UnexpectedToken {
                expected: None,
                found: other,
            }
            .boxed()),
        }
    }

    fn finalize(self: Box<Self>) -> Result<CompiledModule> {
        self.module.ok_or(CompilerError::Unknown { message: "No module declared in this file".into() }.boxed())
    }
}

pub mod import;
pub mod module;
pub mod procedure;
pub mod r#struct;
