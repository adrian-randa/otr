use std::rc::Rc;

use crate::{compiler::{CompilerError, CompilerState, states::CompilerBaseState}, lexer::token::{ParenthesisType, PunctuationToken, Token}, runtime::{RuntimeError, module::Module}};



pub struct CompilerModuleState {
    base: CompilerBaseState,
    module_name: Option<String>,
    in_scope: bool,
    module: Module,
}

impl CompilerModuleState {
    pub fn new(base: CompilerBaseState) -> Self {
        Self {
            base,
            module_name: None,
            in_scope: false,
            module: Module::default()
        }
    }
}

impl CompilerState for CompilerModuleState {
    fn read(mut self, token: Token) -> Result<Box<dyn CompilerState>, crate::compiler::CompilerError> {
        if self.module_name.is_none() {
            if let Token::Identifier(ident) = token {
                self.module_name = Some(ident);
                return Ok(Box::new(self));
            } else {
                return Err(CompilerError {
                    message: format!("Unexpected token! Expected identifier, found {:?}", token)
                });
            }
        }
        if !self.in_scope {
            if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) = token {
                self.in_scope = true;
                return Ok(Box::new(self));
            } else {
                return Err(CompilerError {
                    message: format!("Unexpected token! Expected '{{', found {:?}", token)
                });
            }
        }

        match token {

            Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                self.base.environment.load_module(
                    self.module_name.unwrap(),
                    Rc::new(self.module)
                );
                Ok(Box::new(self.base))
            }

            _ => Err(CompilerError {
                message: format!("Unexpected token! Expected procedure/struct declaration, found {:?}", token)
            })
        }
    }

    fn finalize(self) -> Result<crate::runtime::environment::Environment, crate::compiler::CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
    }
}