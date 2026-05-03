use std::collections::{HashMap, HashSet};

use crate::{error::{CompilerError, context::LineIndexContextDecorator}, states::CompilerBaseState};

use otr_core::{module::{CompiledModule, ImportAddress, ModuleAddress}, error::Result};

pub trait CompilerState {
    fn read(
        self: Box<Self>,
        token: Token,
        compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>>;

    fn finalize(self: Box<Self>) -> Result<CompiledModule>;
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
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            state: Box::new(CompilerBaseState::new()),
        }
    }

    pub fn read(mut self, token: Token, environment: &mut CompilerEnvironment) -> Result<Self> {
        self.state = self.state.read(token, environment)?;
        Ok(self)
    }

    pub fn finalize(self) -> Result<CompiledModule> {
        self.state.finalize()
    }

    pub fn compile(mut self, tokens: impl Iterator<Item = ContextualizedToken>, environment: &mut CompilerEnvironment) -> Result<CompiledModule> {
        for token in tokens {
            let line = token.line_index;
            let token = token.token;

            self = self.read(token, environment)
                .map_err(|error| {
                    LineIndexContextDecorator { error, line }.boxed()
                })
                ?;
        }

        self.finalize()
    }
}

pub struct CompilerEnvironment {
    procedure_ident_map: HashMap<String, ModuleAddress>,
    struct_ident_map: HashMap<String, ModuleAddress>,

    file_read_queue: Vec<ImportAddress>,
    read_modules: HashSet<ImportAddress>,
}

impl CompilerEnvironment {
    pub fn new() -> Self {
        Self {
            procedure_ident_map: Default::default(),
            struct_ident_map: Default::default(),
            
            file_read_queue: Default::default(),
            read_modules: Default::default(),
        }
    }

    pub fn register_procedure_ident(&mut self, address: ModuleAddress) {
        let key = address.get_identifier().to_owned();

        self.procedure_ident_map.insert(key, address);
    }

    pub fn register_struct_ident(&mut self, address: ModuleAddress) {
        let key = address.get_identifier().to_owned();

        self.struct_ident_map.insert(key, address);
    }

    pub fn has_read_module(&self, address: &ImportAddress) -> bool {
        self.read_modules.contains(address)
    }

    pub fn push_file_to_queue(&mut self, address: ImportAddress) {
        if !self.has_read_module(&address) {
            self.file_read_queue.push(address.clone());
            self.read_modules.insert(address);
        }
    }

    pub fn get_next_file(&mut self) -> Option<ImportAddress> {
        self.file_read_queue.pop()
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

pub mod expression_parser;
pub(crate) mod parenthesis;
pub(crate) mod states;
pub(crate) mod procedure;
pub(crate) mod error;
pub mod lexer;

pub use expression_parser::ExpressionParser;
pub use lexer::{Tokenizer, token::{self, *}, fragmenter::{self, *}};