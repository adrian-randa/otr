use crate::{
    CompilerEnvironment, CompilerError, CompilerState, lexer::token::{ParenthesisType, PunctuationToken, Token}, procedure::CompiledProcedureBuilder, states::module::CompilerModuleState
};

use otr_core::{module::{CompiledModule, ModuleAddress}, error::Result};

#[derive(Debug, PartialEq, Eq)]
enum ProcedureSubstate {
    Base,
    FirstIdent,
    PreSecondIdent,
    SecondIdent,
    PreArgument,
    Argument,
    PreInstructions,
    Instructions,
}

enum ProcedureIdentifier {
    Procedure {
        ident: String,
    },
    AssociatedProcedure {
        struct_ident: String,
        procedure_ident: String,
    },
}

pub struct CompilerProcedureState {
    module: CompilerModuleState,
    procedure_identifier: Option<ProcedureIdentifier>,
    procedure: CompiledProcedureBuilder,

    substate: ProcedureSubstate,
}

impl CompilerProcedureState {
    pub fn new(module: CompilerModuleState) -> Self {
        Self {
            module,
            procedure_identifier: None,
            procedure: CompiledProcedureBuilder::new(),

            substate: ProcedureSubstate::Base,
        }
    }
}

impl CompilerState for CompilerProcedureState {
    fn read(
        mut self: Box<Self>,
        token: Token,
        compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        match self.substate {
            ProcedureSubstate::Base => {
                if self.procedure_identifier.is_none() {
                    if let Token::Identifier(ident) = token {
                        self.procedure_identifier = Some(ProcedureIdentifier::Procedure { ident });
                        self.substate = ProcedureSubstate::FirstIdent;
                        Ok(self)
                    } else {
                        Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: token,
                        }
                        .boxed())
                    }
                } else {
                    Err(CompilerError::InvalidDefinition {
                        message: "Procedure already has an identifier!".into(),
                    }
                    .boxed())
                }
            },
            ProcedureSubstate::FirstIdent => match token {
                Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) => {
                    self.substate = ProcedureSubstate::PreArgument;
                    Ok(self)
                }

                Token::Punctuation(PunctuationToken::ThinArrow) => {
                    self.substate = ProcedureSubstate::PreSecondIdent;
                    self.procedure = self.procedure.push_argument_identifier("this".into())?;
                    Ok(self)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("(".into()),
                        found: other,
                    }
                    .boxed())
                }
            },
            ProcedureSubstate::PreSecondIdent => match token {
                Token::Identifier(procedure_ident) => {
                    if let Some(ProcedureIdentifier::Procedure {
                        ident: struct_ident,
                    }) = self.procedure_identifier
                    {
                        self.procedure_identifier =
                            Some(ProcedureIdentifier::AssociatedProcedure {
                                struct_ident,
                                procedure_ident,
                            });
                        self.substate = ProcedureSubstate::SecondIdent;
                        Ok(self)
                    } else {
                        Err(CompilerError::InvalidDefinition {
                            message: "Procedure already associated to a struct!".into(),
                        }
                        .boxed())
                    }
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("Identifier".into()),
                        found: other,
                    }
                    .boxed())
                }
            },
            ProcedureSubstate::SecondIdent => match token {
                Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) => {
                    self.substate = ProcedureSubstate::PreArgument;
                    Ok(self)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("(".into()),
                        found: other,
                    }
                    .boxed())
                }
            },
            ProcedureSubstate::PreArgument => match token {
                Token::Identifier(ident) => {
                    self.procedure = self.procedure.push_argument_identifier(ident)?;
                    self.substate = ProcedureSubstate::Argument;
                    Ok(self)
                }

                Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing)) => {
                    self.substate = ProcedureSubstate::PreInstructions;
                    Ok(self)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("Identifier".into()),
                        found: other,
                    }
                    .boxed())
                }
            },
            ProcedureSubstate::Argument => match token {
                Token::Punctuation(PunctuationToken::Comma) => {
                    self.substate = ProcedureSubstate::PreArgument;
                    Ok(self)
                }

                Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing)) => {
                    self.substate = ProcedureSubstate::PreInstructions;
                    Ok(self)
                }

                _ => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some(", or )".into()),
                        found: token,
                    }
                    .boxed())
                }
            },
            ProcedureSubstate::PreInstructions => {
                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) =
                    token
                {
                    self.substate = ProcedureSubstate::Instructions;
                    Ok(self)
                } else {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("{".into()),
                        found: token,
                    }
                    .boxed())
                }
            }
            ProcedureSubstate::Instructions => {
                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) =
                    token
                    && self.procedure.scope_stack_size() == 0 && !self.procedure.is_scanning() {
                        let procedure = self.procedure.build()?;
                        let name = self.procedure_identifier.ok_or(
                            CompilerError::InvalidDefinition {
                                message: "Missing procedure name!".into(),
                            }
                            .boxed(),
                        )?;

                        match &name {
                            ProcedureIdentifier::Procedure { ident } => {
                                self.module.get_module_mut().insert_procedure(
                                    ident.clone(),
                                    Box::new(procedure),
                                    false,
                                );
                                let module_id = self
                                    .module
                                    .get_name()
                                    .ok_or(
                                        CompilerError::Unknown {
                                            message: "Missing module name!".into(),
                                        }
                                        .boxed(),
                                    )?
                                    .to_owned();
                                let identifier = ident.to_owned();
                                compiler_environment.register_procedure_ident(ModuleAddress::new(
                                    module_id, identifier,
                                ));
                            }
                            ProcedureIdentifier::AssociatedProcedure {
                                struct_ident,
                                procedure_ident,
                            } => {
                                self.module.get_module_mut().insert_associated_procedure(
                                    struct_ident.clone(),
                                    procedure_ident.clone(),
                                    Box::new(procedure),
                                    false,
                                );
                            }
                        }

                        return Ok(Box::new(self.module));
                    }

                self.procedure = self.procedure.read(token, compiler_environment)?;
                Ok(self)
            }
        }
    }

    fn finalize(self: Box<Self>) -> Result<CompiledModule> {
        Err(CompilerError::InvalidDefinition {
            message: "Unfinished module declaration!".into(),
        }
        .boxed())
    }
}
