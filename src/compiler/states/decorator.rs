use crate::{compiler::{CompilerError, CompilerState, states::module::CompilerModuleState}, lexer::token::{PunctuationToken, Token}, runtime::environment::Environment};

pub struct Decorator {
    ident: String,
}

pub struct CompilerDecoratorState {
    module: CompilerModuleState,
    decorators: Vec<Decorator>,
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
    fn read(mut self, token: Token) -> Result<Box<dyn CompilerState>, CompilerError> {
        
        match token {
            
            Token::Punctuation(PunctuationToken::At) => {
                if self.num_decorators > self.decorators.len() {
                    Err(CompilerError{
                        message: format!("Unexpected token! Expected identifier, found {:?}", token)
                    })
                } else {
                    self.num_decorators += 1;
                    Ok(Box::new(self))
                }
            }

            Token::Identifier(ref ident) => {
                if self.decorators.len() >= self.num_decorators {
                    Err(CompilerError{
                        message: format!("Unexpected token! Expected '@', found {:?}", token)
                    })
                } else {
                    self.decorators.push(Decorator { ident: ident.to_string() });
                    Ok(Box::new(self))
                }
            }

            _ => Err(CompilerError{
                message: format!("Unexpected token!")
            })
        }

    }

    fn finalize(self) -> Result<Environment, CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
    }
}