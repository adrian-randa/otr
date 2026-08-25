use crate::{CompilerEnvironment, CompilerError, CompilerState, Module, lexer::token::{KeywordToken, ParenthesisType, PunctuationToken, Token}, states::{CompilerBaseState, import::CompilerImportState, operator_overload::CompilerOperatorOverloadState, procedure::CompilerProcedureState, r#struct::CompilerStructState}
};

use otr_core::{error::Result, expression::Operator, module::CompiledModule};
    
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
    Operator(String, Operator),
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
                    Ok(self)
                } else {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("{".into()),
                        found: token,
                    }
                    .boxed())
                }
            }
            ModuleSubstate::InScope => match token {
                Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                    self.base.module = Some(Module::Compiled(self.module));
                    Ok(Box::new(self.base))
                }

                Token::Keyword(KeywordToken::Import) => {
                    Ok(Box::new(CompilerImportState::new(self)) as Box<dyn CompilerState>)
                }

                Token::Keyword(KeywordToken::Proc) => {
                    Ok(Box::new(CompilerProcedureState::new(self)))
                }

                Token::Keyword(KeywordToken::Struct) => {
                    Ok(Box::new(CompilerStructState::new(self)))
                }

                Token::Keyword(KeywordToken::Export) => {
                    self.substate = ModuleSubstate::Export(ModuleExportSubstate::Base);
                    Ok(self)
                }

                Token::Keyword(KeywordToken::Operator) => {
                    Ok(Box::new(CompilerOperatorOverloadState::new(self)))
                }

                Token::Identifier(_) => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("Procedure/Struct Declaration".into()),
                        found: token,
                    }
                    .boxed())
                }

                _ => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("Procedure/Struct Declaration".into()),
                        found: token,
                    }
                    .boxed())
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
                            if set_procedure_visibility(&mut self.module, &ident, true).is_err() {
                                set_struct_visibility(&mut self.module, &ident, true)?;
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

                        Token::Operator(operator) => {
                            let operator = operator.try_into_core_operator().ok_or_else(||
                                CompilerError::UnexpectedToken { expected: Some("overloadable operator".into()), found: Token::Operator(operator) }.boxed()
                            )?;

                            self.substate = ModuleSubstate::Export(ModuleExportSubstate::Operator(ident, operator));
                            Ok(self)
                        }

                        Token::Punctuation(PunctuationToken::Semicolon) => {
                            if set_procedure_visibility(&mut self.module, &ident, true).is_err() {
                                set_struct_visibility(&mut self.module, &ident, true)?;
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
                            set_associated_precedure_visibility(
                                &mut self.module,
                                &struct_ident,
                                &member_ident,
                                true,
                            )?;
                            self.substate = ModuleSubstate::Export(ModuleExportSubstate::Base);
                            Ok(self)
                        }

                        Token::Punctuation(PunctuationToken::Semicolon) => {
                            set_associated_precedure_visibility(
                                &mut self.module,
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
                    ModuleExportSubstate::Operator(struct_identifier, operator) => match token {

                        Token::Punctuation(PunctuationToken::Semicolon) => {
                            set_operator_visibility(&mut self.module, &struct_identifier, operator, true)?;
                            self.substate = ModuleSubstate::InScope;
                            Ok(self)
                        }

                        Token::Punctuation(PunctuationToken::Comma) => {
                            set_operator_visibility(&mut self.module, &struct_identifier, operator, true)?;
                            self.substate = ModuleSubstate::Export(ModuleExportSubstate::Base);
                            Ok(self)
                        }

                        other => Err(CompilerError::UnexpectedToken {
                            expected: Some("',' or ';'".into()),
                            found: other,
                        }
                        .boxed()),
                    },
                }
            }
        }
    }

    fn finalize(self: Box<Self>) -> Result<Module> {
        Err(CompilerError::InvalidDefinition {
            message: "Unfinished module declaration!".into(),
        }
        .boxed())
    }
}

fn set_procedure_visibility(module: &mut CompiledModule, member_ident: &str, visibility: bool) -> Result<()> {
    if let Some(member) = module.get_procedure_mut(member_ident) {
        member.1 = visibility;
        return Ok(());
    }
    Err(CompilerError::NoSuchMember {
        member_identifier: member_ident.to_string(),
    }
    .boxed())
}

fn set_struct_visibility(module: &mut CompiledModule, member_ident: &str, visibility: bool) -> Result<()> {
    if let Some(member) = module.get_struct_mut(member_ident) {
        member.1 = visibility;
        return Ok(());
    }
    Err(CompilerError::NoSuchMember {
        member_identifier: member_ident.to_string(),
    }
    .boxed())
}

fn set_associated_precedure_visibility(
    module: &mut CompiledModule,
    struct_identifier: &str,
    procedure_identifier: &str,
    visibility: bool,
) -> Result<()> {
    if let Some(member) = module.get_associated_procedure_mut(struct_identifier, procedure_identifier) {
        member.1 = visibility;
        Ok(())
    } else {
        Err(CompilerError::NoSuchMember {
            member_identifier: format!("{struct_identifier}->{procedure_identifier}")
        }.boxed())
    }
}

fn set_operator_visibility(
    module: &mut CompiledModule,
    struct_identifier: &str,
    operator: Operator,
    visibility: bool
) -> Result<()> {
    if let Some(entry) = module.get_operator_mut(struct_identifier, operator) {
        entry.1 = visibility;
        Ok(())
    } else {
        Err(CompilerError::NoSuchMember { member_identifier: format!("{struct_identifier} {operator}") }.boxed())
    }
}