use std::{
    collections::HashMap, path::PathBuf,
};

use crate::{
    compiler::{source_file_reader::{ImportAddress, SourceFileReader}, states::CompilerBaseState}, core::{CompiledObject, module::ModuleAddress}, error::{compiler_error::CompilerError, context::SourceFileContextDecorator}, lexer::{Tokenizer, token::Token}
};

use crate::error::Result;

pub trait CompilerState {
    fn read(
        self: Box<Self>,
        token: Token,
        compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>>;

    fn finalize(self: Box<Self>) -> Result<CompiledObject>;
}

pub trait Decorator {
    fn apply(self: Box<Self>, object: &mut CompiledObject) -> Result<()>;
}

pub trait ExpressionParseEnvironment {
    fn resolve_procedure_identifier(&self, ident: String) -> Result<ModuleAddress>;
    fn resolve_struct_identifier(&self, ident: String) -> Result<ModuleAddress>;
}

pub struct NoExpressionEnvironment;

impl ExpressionParseEnvironment for NoExpressionEnvironment {
    fn resolve_procedure_identifier(&self, ident: String) -> Result<ModuleAddress> {
        Err(CompilerError::InvalidExpression {
            message: format!("Single identifier '{ident}' could not be mapped to procedure!"),
        }
        .boxed())
    }
    fn resolve_struct_identifier(&self, ident: String) -> Result<ModuleAddress> {
        Err(CompilerError::InvalidExpression {
            message: format!("Single identifier '{ident}' could not be mapped to struct!"),
        }
        .boxed())
    }
}

pub struct Compiler {
    state: Box<dyn CompilerState>,
    compiler_environment: CompilerEnvironment,
}

impl Compiler {
    pub fn new(tokenizer: Tokenizer, root_file_path: PathBuf, root_module_ident: String) -> Result<Self> {
        let mut file_reader = SourceFileReader::new(tokenizer, root_file_path);

        file_reader.push_dependency(ImportAddress {
            module_id: root_module_ident,
            path: None
        })?;

        Ok(Self {
            state: Box::new(CompilerBaseState::new()),
            compiler_environment: CompilerEnvironment::new(file_reader),
        })
    }

    pub fn read(mut self, token: Token) -> Result<Self> {
        self.state = self.state.read(token, &mut self.compiler_environment)?;
        Ok(self)
    }

    pub fn finalize(self) -> Result<CompiledObject> {
        let mut object = self.state.finalize()?;

        for decorator in self.compiler_environment.decorators {
            decorator.apply(&mut object)?;
        }

       Ok(object)
    }

    pub fn compile(mut self) -> Result<CompiledObject> {
        while let Some(token) = self.compiler_environment.file_reader.next() {
            let line = token.line_index;
            let column = token.column_index;
            let token = token.token;
            let path = self
                .compiler_environment
                .file_reader
                .get_current_file()?
                .to_path_buf();
            self = match self.read(token) {
                Ok(s) => Ok(s),
                Err(error) => Err(SourceFileContextDecorator {
                    error,
                    path,
                    line,
                    column,
                }
                .boxed()),
            }?;
        }

        self.finalize()
    }
}

pub struct CompilerEnvironment {
    decorators: Vec<Box<dyn Decorator>>,

    procedure_ident_map: HashMap<String, ModuleAddress>,
    struct_ident_map: HashMap<String, ModuleAddress>,

    file_reader: SourceFileReader,
}

impl CompilerEnvironment {
    pub(crate) fn new(file_reader: SourceFileReader) -> Self {
        Self {
            decorators: Vec::new(),
            procedure_ident_map: Default::default(),
            struct_ident_map: Default::default(),
            file_reader,
        }
    }

    pub fn push_decorator(&mut self, decorator: Box<dyn Decorator>) {
        self.decorators.push(decorator);
    }

    pub fn get_file_reader(&self) -> &SourceFileReader {
        &self.file_reader
    }

    pub fn get_file_reader_mut(&mut self) -> &mut SourceFileReader {
        &mut self.file_reader
    }

    pub fn register_procedure_ident(&mut self, address: ModuleAddress) {
        let key = address.get_identifier().to_owned();

        self.procedure_ident_map.insert(key, address);
    }

    pub fn register_struct_ident(&mut self, address: ModuleAddress) {
        let key = address.get_identifier().to_owned();

        self.struct_ident_map.insert(key, address);
    }
}

impl ExpressionParseEnvironment for CompilerEnvironment {
    fn resolve_procedure_identifier(&self, ident: String) -> Result<ModuleAddress> {
        self.procedure_ident_map
            .get(&ident)
            .ok_or(
                CompilerError::InvalidExpression {
                    message: format!(
                        "Single identifier '{ident}' could not be mapped to a procedure!"
                    ),
                }
                .boxed(),
            )
            .map(|address| address.to_owned())
    }

    fn resolve_struct_identifier(&self, ident: String) -> Result<ModuleAddress> {
        self.struct_ident_map
            .get(&ident)
            .ok_or(
                CompilerError::InvalidExpression {
                    message: format!(
                        "Single identifier '{ident}' could not be mapped to a struct!"
                    ),
                }
                .boxed(),
            )
            .map(|address| address.to_owned())
    }
}

pub mod decorators;
pub mod expression_parser;
pub mod source_file_reader;
pub mod parenthesis;
pub mod states;
pub mod procedure;