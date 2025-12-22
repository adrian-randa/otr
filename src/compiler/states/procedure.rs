use std::fmt::Arguments;

use crate::{compiler::{Compiler, CompilerEnvironment, CompilerError, CompilerState, decorators::EntrypointDecorator, states::{decorator::{self, RawDecorator}, module::CompilerModuleState}}, lexer::token::{ParenthesisType, PunctuationToken, Token}, runtime::{ModuleAddress, procedures::CompiledProcedureBuilder}};

#[derive(Debug, PartialEq, Eq)]
enum ProcedureSubstate {
    Ident,
    PreArgument,
    Argument,
    PreInstructions,
    Instructions,
}

pub struct CompilerProcedureState {
    module: CompilerModuleState,
    decorators: Vec<RawDecorator>,
    name: Option<String>,
    procedure: CompiledProcedureBuilder,

    substate: ProcedureSubstate,
}

impl CompilerProcedureState {
    pub fn new(module: CompilerModuleState, decorators: Vec<RawDecorator>) -> Self {
        Self {
            module, decorators,
            name: None,
            procedure: CompiledProcedureBuilder::new(),

            substate: ProcedureSubstate::Ident,
        }
    }
}

impl CompilerState for CompilerProcedureState {
    fn read(mut self: Box<Self>, token: Token, compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>, crate::compiler::CompilerError> {
        if self.name.is_none() {
            if let Token::Identifier(ident) = token {
                self.name = Some(ident);
                return Ok(self);
            } else {
                return Err(CompilerError {
                    message: format!("Unexpected token! Expected identifier, found {:?}", token)
                });
            }
        }

        match self.substate {
            ProcedureSubstate::Ident => {
                if let Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) = token {
                    self.substate = ProcedureSubstate::PreArgument;
                    return Ok(self);
                } else {
                    Err(CompilerError {
                        message: format!("Unexpected token! Expected '(', found {:?}", token)
                    })
                }
            }
            ProcedureSubstate::PreArgument => {
                match token {
                    Token::Identifier(ident) => {
                        self.procedure = self.procedure.push_argument_identifier(ident);
                        self.substate = ProcedureSubstate::Argument;
                        return Ok(self)
                    }

                    Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing)) => {
                        self.substate = ProcedureSubstate::PreInstructions;
                        return Ok(self);
                    }
                    

                    other => {
                        return Err(CompilerError {
                            message: format!("Unexpected token! Expected identifier, found {:?}", other)
                        });
                    }
                }
            },
            ProcedureSubstate::Argument => {
                match token {
                    Token::Punctuation(PunctuationToken::Comma) => {
                        self.substate = ProcedureSubstate::PreArgument;
                        return Ok(self);
                    }

                    Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing)) => {
                        self.substate = ProcedureSubstate::PreInstructions;
                        return Ok(self)
                    }

                    _ => {
                        return Err(CompilerError{
                            message: format!("Unexpected token! Expected ',' or ')', found {:?}", token)
                        });
                    }
                }
            }
            ProcedureSubstate::PreInstructions => {
                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) = token {
                    self.substate = ProcedureSubstate::Instructions;
                    return Ok(self);
                } else {
                    return Err(CompilerError{
                        message: format!("Unexpected token! Expected '{{', found {:?}", token)
                    });
                }
            },
            ProcedureSubstate::Instructions => {
                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) = token {
                    if self.procedure.scope_stack_size() == 0 {
                        let procedure = self.procedure.build()?;
                        let name = self.name.ok_or(CompilerError {
                            message: "Missing procedure name!".into()
                        })?;

                        self.module.get_module_mut().insert_procedure(
                            name.clone(),
                            Box::new(procedure),
                            false
                        );

                        for decorator in self.decorators {
                            match decorator.get_ident() as &str {
                                "entrypoint" => {
                                    compiler_environment.push_decorator(
                                        Box::new(EntrypointDecorator::new(
                                            ModuleAddress::new(
                                                self.module
                                                    .get_name().ok_or(CompilerError {
                                                        message: "Contained module has no name!".into()
                                                    })?.to_owned(),
                                                    name.clone()
                                                )
                                        ))
                                    );
                                }

                                other => {return Err(CompilerError {
                                    message: format!("Unsupported decorator '{}'!", other)
                                })}
                            }
                        }

                        return Ok(Box::new(self.module))
                    }
                }

                self.procedure = self.procedure.read(token)?;
                Ok(self)
            },
        }
    }

    fn finalize(self: Box<Self>) -> Result<crate::runtime::environment::Environment, crate::compiler::CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
    }
}