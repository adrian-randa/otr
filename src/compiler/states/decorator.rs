use crate::{compiler::{Compiler, CompilerEnvironment, CompilerError, CompilerState, states::{module::CompilerModuleState, procedure::CompilerProcedureState}}, lexer::token::{KeywordToken, PunctuationToken, Token}, runtime::environment::Environment};

#[derive(Clone)]
pub struct RawDecorator {
    ident: String,
}

impl RawDecorator {
    pub fn get_ident(&self) -> &String {
        &self.ident
    }
}

pub struct CompilerDecoratorState {
    module: CompilerModuleState,
    decorators: Vec<RawDecorator>,
    num_decorators: usize,
}

impl CompilerDecoratorState {
    pub fn new(module: CompilerModuleState) -> Self {
        Self {
            module,
            decorators: Vec::new(),
            num_decorators: 1,
        }
    }
}

impl CompilerState for CompilerDecoratorState {
    fn read(mut self: Box<Self>, token: Token, _compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>, CompilerError> {
        
        match token {
            
            Token::Punctuation(PunctuationToken::At) => {
                if self.num_decorators > self.decorators.len() {
                    Err(CompilerError{
                        message: format!("Unexpected token! Expected identifier, found {:?}", token)
                    })
                } else {
                    self.num_decorators += 1;
                    Ok(self)
                }
            }

            Token::Identifier(ref ident) => {
                if self.decorators.len() >= self.num_decorators {
                    Err(CompilerError{
                        message: format!("Unexpected token! Expected '@', found {:?}", token)
                    })
                } else {
                    self.decorators.push(RawDecorator { ident: ident.to_string() });
                    Ok(self)
                }
            }

            Token::Keyword(KeywordToken::Proc) => {
                return Ok(Box::new(
                    CompilerProcedureState::new(
                        self.module,
                        self.decorators
                    )
                ));
            }

            _ => Err(CompilerError{
                message: format!("Unexpected token!")
            })
        }

    }

    fn finalize(self: Box<Self>) -> Result<Environment, CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
    }
}