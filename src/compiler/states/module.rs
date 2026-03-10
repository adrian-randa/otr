use std::rc::Rc;

use crate::{
    compiler::{
        states::{
            decorator::CompilerDecoratorState, procedure::CompilerProcedureState,
            r#struct::CompilerStructState, CompilerBaseState,
        }, CompilerEnvironment, CompilerError, CompilerState,
    },
    error::context::HintContextDecorator,
    lexer::token::{KeywordToken, ParenthesisType, PunctuationToken, Token},
    runtime::module::{CompiledModule, Module},
};

use crate::error::Result;

#[derive(Debug, PartialEq, Eq)]
enum ModuleSubstate {
    PreScope,
    InScope,
    Export(ModuleExportSubstate),
}

#[derive(Debug, PartialEq, Eq)]
enum ModuleExportSubstate {
    Base,
    SingleIdent(String),
    Arrow {
        struct_ident: String,
    },
    AssociatedProcedure {
        struct_ident: String,
        member_ident: String,
    },
}

pub struct CompilerModuleState {
    base: CompilerBaseState,
    module_name: Option<String>,
    substate: ModuleSubstate,
    module: CompiledModule,
}

impl CompilerModuleState {
    pub fn new(base: CompilerBaseState) -> Self {
        Self {
            base,
            module_name: None,
            substate: ModuleSubstate::PreScope,
            module: CompiledModule::default(),
        }
    }

    pub fn get_module_mut(&mut self) -> &mut CompiledModule {
        &mut self.module
    }

    pub fn get_name(&self) -> Option<&String> {
        self.module_name.as_ref()
    }
}

impl CompilerState for CompilerModuleState {
    fn read(
        mut self: Box<Self>,
        token: Token,
        _compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        match self.substate {
            ModuleSubstate::PreScope => {
                if self.module_name.is_none() {
                    if let Token::Identifier(ident) = token {
                        self.module_name = Some(ident);
                        return Ok(self);
                    } else {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: token,
                        }
                        .boxed());
                    }
                }

                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) =
                    token
                {
                    self.substate = ModuleSubstate::InScope;
                    return Ok(self);
                } else {
                    return Err(CompilerError::UnexpectedToken {
                        expected: Some("{".into()),
                        found: token,
                    }
                    .boxed());
                }
            }
            ModuleSubstate::InScope => match token {
                Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                    self.base
                        .environment
                        .load_module(self.module_name.unwrap(), Rc::new(self.module));
                    Ok(Box::new(self.base))
                }

                Token::Keyword(KeywordToken::Proc) => {
                    return Ok(Box::new(CompilerProcedureState::new(*self, Vec::new())));
                }

                Token::Keyword(KeywordToken::Struct) => {
                    return Ok(Box::new(CompilerStructState::new(*self)));
                }

                Token::Punctuation(PunctuationToken::At) => {
                    return Ok(Box::new(CompilerDecoratorState::new(*self)));
                }

                Token::Keyword(KeywordToken::Export) => {
                    self.substate = ModuleSubstate::Export(ModuleExportSubstate::Base);
                    return Ok(self);
                }

                Token::Identifier(_) => {
                    return Err(HintContextDecorator {
                        error: CompilerError::UnexpectedToken {
                            expected: Some("Procedure/Struct Declaration".into()),
                            found: token,
                        }
                        .boxed(),

                        message: "Specify what you want to declare: Use 'proc' or 'struct'!".into(),
                    }
                    .boxed())
                }

                _ => {
                    return Err(CompilerError::UnexpectedToken {
                        expected: Some("Procedure/Struct Declaration".into()),
                        found: token,
                    }
                    .boxed());
                }
            },
            ModuleSubstate::Export(substate) => {
                match substate {
                    ModuleExportSubstate::Base => match token {
                        Token::Identifier(ident) => {
                            self.substate =
                                ModuleSubstate::Export(ModuleExportSubstate::SingleIdent(ident));
                            Ok(self)
                        }

                        other => Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: other,
                        }
                        .boxed()),
                    },
                    ModuleExportSubstate::SingleIdent(ident) => match token {
                        Token::Punctuation(PunctuationToken::Comma) => {
                            if let Err(_) = self.module.set_procedure_visibility(&ident, true) {
                                self.module.set_struct_visibility(&ident, true)?;
                            }
                            self.substate = ModuleSubstate::Export(ModuleExportSubstate::Base);
                            Ok(self)
                        }

                        Token::Punctuation(PunctuationToken::ThinArrow) => {
                            self.substate = ModuleSubstate::Export(ModuleExportSubstate::Arrow {
                                struct_ident: ident,
                            });
                            Ok(self)
                        }

                        Token::Punctuation(PunctuationToken::Semicolon) => {
                            if let Err(_) = self.module.set_procedure_visibility(&ident, true) {
                                self.module.set_struct_visibility(&ident, true)?;
                            }
                            self.substate = ModuleSubstate::InScope;
                            Ok(self)
                        }

                        other => Err(CompilerError::UnexpectedToken {
                            expected: Some("',', ';' or '->'".into()),
                            found: other,
                        }
                        .boxed()),
                    },
                    ModuleExportSubstate::Arrow { struct_ident } => match token {
                        Token::Identifier(member_ident) => {
                            self.substate =
                                ModuleSubstate::Export(ModuleExportSubstate::AssociatedProcedure {
                                    struct_ident,
                                    member_ident,
                                });
                            Ok(self)
                        }

                        other => Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: other,
                        }
                        .boxed()),
                    },
                    ModuleExportSubstate::AssociatedProcedure {
                        struct_ident,
                        member_ident,
                    } => match token {
                        Token::Punctuation(PunctuationToken::Comma) => {
                            self.module.set_associated_precedure_visibility(
                                &struct_ident,
                                &member_ident,
                                true,
                            )?;
                            self.substate = ModuleSubstate::Export(ModuleExportSubstate::Base);
                            Ok(self)
                        }

                        Token::Punctuation(PunctuationToken::Semicolon) => {
                            self.module.set_associated_precedure_visibility(
                                &struct_ident,
                                &member_ident,
                                true,
                            )?;
                            self.substate = ModuleSubstate::InScope;
                            Ok(self)
                        }

                        other => Err(CompilerError::UnexpectedToken {
                            expected: Some("',' or ';'".into()),
                            found: other,
                        }
                        .boxed()),
                    },
                }
                /* match token {
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
                } */
            }
        }
    }

    fn finalize(self: Box<Self>) -> Result<crate::runtime::environment::Environment> {
        Err(CompilerError::InvalidDefinition {
            message: "Unfinished module declaration!".into(),
        }
        .boxed())
    }
}
