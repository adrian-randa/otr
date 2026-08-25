use otr_core::{vec_map::VecMap, Result};
use otr_ffi::external::{ExternalFunction, ExternalModule};

use crate::{CompilerState, KeywordToken, Module, ParenthesisType::{Closing, Opening}, PunctuationToken::{Comma, CurlyBraces, Parenthesis, Semicolon}, Token::{self, Keyword, Punctuation}, error::CompilerError, states::CompilerBaseState};



pub(crate) enum CompilerExternalModuleState {
    Base{
        base: CompilerBaseState,
    },
    Module {
        base: CompilerBaseState,
    },
    Identifier {
        base: CompilerBaseState,
        module_identifier: String,
    },
    InsideScope {
        base: CompilerBaseState,
        module_identifier: String,
        procedures: VecMap<String, ExternalFunction>,
        state: ExternalProcDefinitionState,
    }
}

impl CompilerState for CompilerExternalModuleState {
    fn read(
        mut self: Box<Self>,
        token: Token,
        _compiler_environment: &mut crate::CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        use CompilerExternalModuleState::*;
        match (*self, token) {
            (Base { base }, Keyword(KeywordToken::Module)) => {
                *self = Module { base };
                Ok(self)
            }
            (Base { base: _ }, found) => {
                Err(CompilerError::UnexpectedToken { expected: Some("'module'".into()), found }.boxed())
            }
            (Module { base }, Token::Identifier(module_identifier)) => {
                *self = Identifier { base, module_identifier };
                Ok(self)
            }
            (Module { base: _ }, found) => {
                Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found }.boxed())
            }
            (Identifier { base, module_identifier }, Punctuation(CurlyBraces(Opening))) => {
                *self = InsideScope { base, module_identifier, procedures: VecMap::default(), state: ExternalProcDefinitionState::Base };
                Ok(self)
            }
            (Identifier { base: _, module_identifier: _ }, found) => {
                Err(CompilerError::UnexpectedToken { expected: Some("'{'".into()), found }.boxed())
            }
            (InsideScope { mut base, module_identifier, procedures, state }, Punctuation(CurlyBraces(Closing))) => {
                if !matches!(state, ExternalProcDefinitionState::Base) {
                    return Err(CompilerError::InvalidDefinition { message: "Incomplete external procedure declaration!".into() }.boxed());
                }
                base.module = Some(crate::Module::External(ExternalModule { library_name: module_identifier, functions: procedures }));
                Ok(Box::new(base))
            }
            (InsideScope { base, module_identifier, mut procedures, state }, token) => {
                let (state, emitted) = state.read(token)?;
                if let Some((identifier, arguments)) = emitted {
                    procedures.insert(identifier, ExternalFunction { arguments });
                }
                *self = Self::InsideScope { base, module_identifier, procedures, state };
                Ok(self)
            }

            _ => todo!()
        }
    }

    fn finalize(self: Box<Self>) -> Result<Module> {
        Err(CompilerError::InvalidDefinition {
            message: "Unfinished module declaration!".into(),
        }
        .boxed())
    }
}

pub(crate) enum ExternalProcDefinitionState {
    Base,
    Proc,
    Identifier {
        proc_identifier: String,
    },
    InsideParenthesis {
        proc_identifier: String,
        arg_identifiers: Vec<String>,
    },
    InsideParenthesisBeforeComma {
        proc_identifier: String,
        arg_identifiers: Vec<String>,
    },
    AfterScope,
}

impl ExternalProcDefinitionState {
    fn read(mut self, token: Token) -> Result<(Self, Option<(String, Vec<String>)>)> {
        use ExternalProcDefinitionState::*;

        match (self, token) {
            (Base, Keyword(KeywordToken::Proc)) => {
                self = Proc;
                Ok((self, None))
            }
            (Base, found) => Err(CompilerError::UnexpectedToken { expected: Some("'Proc'".into()), found }.boxed()),
            (Proc, Token::Identifier(proc_identifier)) => {
                self = Identifier { proc_identifier };
                Ok((self, None))
            }
            (Proc, found) => Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found }.boxed()),
            (Identifier { proc_identifier }, Punctuation(Parenthesis(Opening))) => {
                self = InsideParenthesis { proc_identifier, arg_identifiers: Vec::new() };
                Ok((self, None))
            }
            (Identifier { proc_identifier: _ }, found) => Err(CompilerError::UnexpectedToken { expected: Some("'('".into()), found }.boxed()),
            (InsideParenthesis { proc_identifier, mut arg_identifiers }, Token::Identifier(ident)) => {
                arg_identifiers.push(ident);
                self = InsideParenthesisBeforeComma { proc_identifier, arg_identifiers };
                Ok((self, None))
            }
            (InsideParenthesis { proc_identifier, arg_identifiers }, Punctuation(Parenthesis(Closing))) => {
                self = AfterScope;
                Ok((self, Some((proc_identifier, arg_identifiers))))
            }
            (InsideParenthesis { proc_identifier: _, arg_identifiers: _ }, found) => {
                Err(CompilerError::UnexpectedToken { expected: Some("argument(s) or ')'".into()), found }.boxed())
            }
            (InsideParenthesisBeforeComma { proc_identifier, arg_identifiers }, Punctuation(Comma)) => {
                self = InsideParenthesis { proc_identifier, arg_identifiers };
                Ok((self, None))
            }
            (InsideParenthesisBeforeComma { proc_identifier, arg_identifiers }, Punctuation(Parenthesis(Closing))) => {
                self = AfterScope;
                Ok((self, Some((proc_identifier, arg_identifiers))))
            }
            (InsideParenthesisBeforeComma { proc_identifier: _, arg_identifiers: _ }, found) => {
                Err(CompilerError::UnexpectedToken { expected: Some("',' or ')'".into()), found }.boxed())
            }
            (AfterScope, Punctuation(Semicolon)) => {
                self = Base;
                Ok((self, None))
            }
            (AfterScope, found) => Err(CompilerError::UnexpectedToken { expected: Some("';'".into()), found }.boxed()),
        }
    }
}