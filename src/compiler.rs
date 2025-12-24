use std::{collections::HashSet, str::FromStr};

use crate::{compiler::{file_reader::FileReader, states::CompilerBaseState}, lexer::{FragmentStream, Tokenizer, token::Token}, runtime::{RuntimeObject, environment::Environment}};

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
    tokenizer: Tokenizer,
    state: Box<dyn CompilerState>,
    compiler_environment: CompilerEnvironment,
}

impl Compiler {
    pub fn new(file_reader: FileReader) -> Self {
        Self {
            tokenizer: Tokenizer::default(),
            state: Box::new(CompilerBaseState::new()),
            compiler_environment: CompilerEnvironment::new(file_reader)
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

    pub fn compile(mut self) -> Result<RuntimeObject, CompilerError> {
        while let Some(next_module) = self.compiler_environment.file_reader.dequeue()? {
            let fragments = FragmentStream::from_str(&next_module)
                .map_err(|err| CompilerError {
                    message: format!("Fragmentation error: {:?}", err)
                })?;
            
            let tokens = self.tokenizer.tokenize(fragments)
                .map_err(|err| CompilerError {
                    message: format!("Tokenization error: {:?}", err)
                })?;
            
            for token in tokens {
                self = self.read(token)?;
            }
        }

        self.finalize()
    }
}

pub struct CompilerEnvironment {
    decorators: Vec<Box<dyn Decorator>>,

    file_reader: FileReader,
}

impl CompilerEnvironment {
    pub(crate) fn new(file_reader: FileReader) -> Self {
        Self {
            decorators: Vec::new(),
            file_reader,
        }
    }

    pub fn push_decorator(&mut self, decorator: Box<dyn Decorator>) {
        self.decorators.push(decorator);
    }

    pub fn get_file_reader(&self) -> &FileReader {
        &self.file_reader
    }

    pub fn get_file_reader_mut(&mut self) -> &mut FileReader {
        &mut self.file_reader
    }
}

pub mod states;
pub mod expression_parser;
pub mod decorators;
pub mod file_reader;