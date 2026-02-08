use std::rc::Rc;

use crate::{compiler::{Compiler, CompilerEnvironment, CompilerError, CompilerState, states::{CompilerBaseState, decorator::CompilerDecoratorState, procedure::CompilerProcedureState, r#struct::CompilerStructState}}, error::context::HintContextDecorator, lexer::token::{KeywordToken, ParenthesisType, PunctuationToken, Token}, runtime::module::Module};

use crate::error::Result;

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
    fn read(mut self: Box<Self>, token: Token, _compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>> {

        match self.substate {
            ModuleSubstate::PreScope => {
                if self.module_name.is_none() {
                    if let Token::Identifier(ident) = token {
                        self.module_name = Some(ident);
                        return Ok(self);
                    } else {
                        return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: token }.boxed());
                    }
                }

                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) = token {
                    self.substate = ModuleSubstate::InScope;
                    return Ok(self);
                } else {
                    return Err(CompilerError::UnexpectedToken { expected: Some("{".into()), found: token }.boxed());
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

                    Token::Identifier(_) => {
                        return Err(HintContextDecorator {
                            error: CompilerError::UnexpectedToken {
                                expected: Some("Procedure/Struct Declaration".into()),
                                found: token,
                            }.boxed(),

                            message: "Specify what you want to declare: Use 'proc' or 'struct'!".into()
                        }.boxed())
                    }

                    _ => {
                        return Err(CompilerError::UnexpectedToken { expected: Some("Procedure/Struct Declaration".into()), found: token }.boxed());
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
                        return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: other }.boxed());
                    }
                }
            },
        }

        
    }

    fn finalize(self: Box<Self>) -> Result<crate::runtime::environment::Environment> {
        Err(CompilerError::InvalidDefinition {
            message: "Unfinished module declaration!".into()
        }.boxed())
    }
}