use crate::{compiler::states::CompilerBaseState, lexer::token::Token, runtime::{RuntimeObject, environment::Environment}};

#[derive(Debug)]
pub struct CompilerError {
    pub message: String,
}

pub trait CompilerState {
    fn read(self: Box<Self>, token: Token, compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>, CompilerError>;

    fn finalize(self: Box<Self>) -> Result<Environment, CompilerError>;
}

pub trait Decorator {
    fn apply(self: Box<Self>, runtime_object: &mut RuntimeObject) -> Result<(), CompilerError>;
}

pub struct Compiler {
    state: Box<dyn CompilerState>,
    compiler_environment: CompilerEnvironment,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            state: Box::new(CompilerBaseState::new()),
            compiler_environment: CompilerEnvironment::new()
        }
    }

    pub fn read(mut self, token: Token) -> Result<Self, CompilerError> {
        self.state = self.state.read(token, &mut self.compiler_environment)?;
        Ok(self)
    }

    pub fn finalize(self) -> Result<RuntimeObject, CompilerError> {
        let mut runtime_object = RuntimeObject::new();

        runtime_object.base_environement = self.state.finalize()?;

        for decorator in self.compiler_environment.decorators {
            decorator.apply(&mut runtime_object)?;
        }

        Ok(runtime_object)
    }
}

pub struct CompilerEnvironment {
    decorators: Vec<Box<dyn Decorator>>,
}

impl CompilerEnvironment {
    pub(crate) fn new() -> Self {
        Self {
            decorators: Vec::new(),
        }
    }

    pub fn push_decorator(&mut self, decorator: Box<dyn Decorator>) {
        self.decorators.push(decorator);
    }
}

pub mod states;
pub mod expression_parser;
pub mod decorators;