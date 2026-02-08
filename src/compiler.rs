use std::{collections::HashSet, str::FromStr};

use crate::{compiler::{file_reader::FileReader, states::CompilerBaseState}, error::{compiler_error::CompilerError, context::SourceFileContextDecorator}, lexer::{FragmentStream, Tokenizer, token::Token}, runtime::{RuntimeObject, environment::Environment}};

use crate::error::Result;

pub trait CompilerState {
    fn read(self: Box<Self>, token: Token, compiler_environment: &mut CompilerEnvironment) -> Result<Box<dyn CompilerState>>;

    fn finalize(self: Box<Self>) -> Result<Environment>;
}

pub trait Decorator {
    fn apply(self: Box<Self>, runtime_object: &mut RuntimeObject) -> Result<()>;
}

pub struct Compiler {
    state: Box<dyn CompilerState>,
    compiler_environment: CompilerEnvironment,
}

impl Compiler {
    pub fn new(file_reader: FileReader) -> Self {
        Self {
            state: Box::new(CompilerBaseState::new()),
            compiler_environment: CompilerEnvironment::new(file_reader)
        }
    }

    pub fn read(mut self, token: Token) -> Result<Self> {
        self.state = self.state.read(token, &mut self.compiler_environment)?;
        Ok(self)
    }

    pub fn finalize(self) -> Result<RuntimeObject> {
        let mut runtime_object = RuntimeObject::new();

        runtime_object.base_environement = self.state.finalize()?;

        for decorator in self.compiler_environment.decorators {
            decorator.apply(&mut runtime_object)?;
        }

        Ok(runtime_object)
    }

    pub fn compile(mut self) -> Result<RuntimeObject> {
        while let Some(token) = self.compiler_environment.file_reader.next() {
            let line = token.line_index;
            let column = token.column_index;
            let token = token.token;
            let path = self.compiler_environment.file_reader.get_current_file()?.clone();
            self = match self.read(token) {
                Ok(s) => Ok(s),
                Err(error) => Err(SourceFileContextDecorator {
                    error,
                    path,
                    line,
                    column,
                }.boxed())
            }?;
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