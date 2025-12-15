use std::fmt::Arguments;

use crate::{compiler::{CompilerError, CompilerState, states::{decorator::Decorator, module::CompilerModuleState}}, lexer::token::{ParenthesisType, PunctuationToken, Token}, runtime::procedures::CompiledProcedureBuilder};

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
    decorators: Vec<Decorator>,
    name: Option<String>,
    procedure: CompiledProcedureBuilder,

    substate: ProcedureSubstate,
}

impl CompilerProcedureState {
    pub fn new(module: CompilerModuleState, decorators: Vec<Decorator>) -> Self {
        Self {
            module, decorators,
            name: None,
            procedure: CompiledProcedureBuilder::new(),

            substate: ProcedureSubstate::Ident,
        }
    }
}

impl CompilerState for CompilerProcedureState {
    fn read(mut self, token: Token) -> Result<Box<dyn CompilerState>, crate::compiler::CompilerError> {
        if self.name.is_none() {
            if let Token::Identifier(ident) = token {
                self.name = Some(ident);
                return Ok(Box::new(self));
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
                    return Ok(Box::new(self));
                } else {
                    Err(CompilerError {
                        message: format!("Unexpected token! Expected '(', found {:?}", token)
                    })
                }
            }
            ProcedureSubstate::PreArgument => {
                if let Token::Identifier(ident) = token {
                    self.procedure = self.procedure.push_argument_identifier(ident);
                    self.substate = ProcedureSubstate::Argument;
                    return Ok(Box::new(self))
                } else {
                    return Err(CompilerError {
                        message: format!("Unexpected token! Expected identifier, found {:?}", token)
                    });
                }
            },
            ProcedureSubstate::Argument => {
                match token {
                    Token::Punctuation(PunctuationToken::Comma) => {
                        self.substate = ProcedureSubstate::PreArgument;
                        return Ok(Box::new(self));
                    }

                    Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing)) => {
                        self.substate = ProcedureSubstate::PreInstructions;
                        return Ok(Box::new(self))
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
                    return Ok(Box::new(self));
                } else {
                    return Err(CompilerError{
                        message: format!("Unexpected token! Expected '{{', found {:?}", token)
                    });
                }
            },
            ProcedureSubstate::Instructions => todo!(),
        }
    }

    fn finalize(self) -> Result<crate::runtime::environment::Environment, crate::compiler::CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
    }
}