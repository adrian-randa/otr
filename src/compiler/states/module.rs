use std::rc::Rc;

use crate::{compiler::{Compiler, CompilerEnvironment, CompilerError, CompilerState, states::{CompilerBaseState, decorator::CompilerDecoratorState, procedure::CompilerProcedureState, r#struct::CompilerStructState}}, lexer::token::{KeywordToken, ParenthesisType, PunctuationToken, Token}, runtime::{RuntimeError, module::Module}};

#[derive(Debug, PartialEq, Eq)]
enum ModuleSubstate {
    PreScope,
    InScope,
    Export,
}

pub struct CompilerModuleState {
    base: CompilerBaseState,
    module_name: Option<String>,
    substate: ModuleSubstate,
    module: Module,
}

impl CompilerModuleState {
    pub fn new(base: CompilerBaseState) -> Self {
        Self {
            base,
            module_name: None,
            substate: ModuleSubstate::PreScope,
            module: Module::default()
        }
    }

    pub fn get_module_mut(&mut self) -> &mut Module {
        &mut self.module
    }

    pub fn get_name(&self) -> Option<&String> {
        self.module_name.as_ref()
    }
}

impl CompilerState for CompilerModuleState {
    fn read(mut self: Box<Self>, token: Token, _compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>, crate::compiler::CompilerError> {

        match self.substate {
            ModuleSubstate::PreScope => {
                if self.module_name.is_none() {
                    if let Token::Identifier(ident) = token {
                        self.module_name = Some(ident);
                        return Ok(self);
                    } else {
                        return Err(CompilerError {
                            message: format!("Unexpected token! Expected identifier, found {:?}", token)
                        });
                    }
                }

                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) = token {
                    self.substate = ModuleSubstate::InScope;
                    return Ok(self);
                } else {
                    return Err(CompilerError {
                        message: format!("Unexpected token! Expected '{{', found {:?}", token)
                    });
                }
            },
            ModuleSubstate::InScope => {
                match token {
                    Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                        self.base.environment.load_module(
                            self.module_name.unwrap(),
                            Rc::new(self.module)
                        );
                        Ok(Box::new(self.base))
                    }

                    Token::Keyword(KeywordToken::Proc) => {
                        return Ok(Box::new(CompilerProcedureState::new(*self, Vec::new())));
                    }

                    Token::Keyword(KeywordToken::Struct) => {
                        return Ok(Box::new(CompilerStructState::new(*self)));
                    }

                    Token::Punctuation(PunctuationToken::At) => {
                        return Ok(Box::new(
                            CompilerDecoratorState::new(*self)
                        ));
                    }

                    Token::Keyword(KeywordToken::Export) => {
                        self.substate = ModuleSubstate::Export;
                        return Ok(self);
                    }

                    _ => {
                        return Err(CompilerError {
                            message: format!("Unexpected token! Expected procedure/struct declaration, found {:?}", token)
                        });
                    }
                }
            },
            ModuleSubstate::Export => {
                match token {
                    Token::Punctuation(PunctuationToken::Comma) => {
                        return Ok(self);
                    }

                    Token::Identifier(ident) => {
                        self.module.set_member_visibility(&ident, true)?;
                        return Ok(self);
                    }

                    Token::Punctuation(PunctuationToken::Semicolon) => {
                        self.substate = ModuleSubstate::InScope;
                        return Ok(self);
                    }

                    other => {
                        return Err(CompilerError {
                            message: format!("Unexpected token. Expected identifier, found {:?}!", other)
                        });
                    }
                }
            },
        }

        
    }

    fn finalize(self: Box<Self>) -> Result<crate::runtime::environment::Environment, crate::compiler::CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
    }
}