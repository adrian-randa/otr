use std::convert::Infallible;

use crate::{lexer::token::Token, runtime::environment::Environment};

#[derive(Debug)]
pub struct CompilerError {
    pub message: String,
}

pub trait CompilerState {
    fn read(self, token: Token) -> Result<Box<dyn CompilerState>, CompilerError>;

    fn finalize(self) -> Result<Environment, CompilerError>;
}

pub mod states;
pub mod expression_parser;