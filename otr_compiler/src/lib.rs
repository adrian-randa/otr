use std::collections::{HashMap, HashSet};

use crate::{error::{CompilerError, context::LineIndexContextDecorator}, states::CompilerBaseState};

use otr_core::{module::{CompiledModule, ImportAddress, ModuleAddress}, error::Result};

#[derive(Debug, Clone)]
pub enum Module {
    Compiled(CompiledModule),
    External(ExternalModule),
}

pub trait CompilerState {
    fn read(
        self: Box<Self>,
        token: Token,
        compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>>;

    fn finalize(self: Box<Self>) -> Result<Module>;
}

pub trait ExpressionParseEnvironment: std::fmt::Debug {
    fn resolve_procedure_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress>;
    fn resolve_struct_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress>;
    fn resolve_variable_ident(&self, ident: &dyn AsRef<str>) -> Result<usize>;
}

#[derive(Debug)]
pub struct NoExpressionEnvironment;

impl ExpressionParseEnvironment for NoExpressionEnvironment {
    fn resolve_procedure_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        Err(CompilerError::InvalidExpression {
            message: format!("Single identifier '{}' could not be mapped to procedure!", ident.as_ref()),
        }
        .boxed())
    }
    fn resolve_struct_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        Err(CompilerError::InvalidExpression {
            message: format!("Single identifier '{}' could not be mapped to struct!", ident.as_ref()),
        }
        .boxed())
    }
    
    fn resolve_variable_ident(&self, ident: &dyn AsRef<str>) -> Result<usize> {
        Err(CompilerError::InvalidExpression {
            message: format!("Single identifier '{}' could not be mapped to variable!", ident.as_ref()),
        }
        .boxed())
    }
}

#[derive(Debug)]
pub(crate) struct FallbackExpressionParseEnvironemnt<'a> {
    main: &'a dyn ExpressionParseEnvironment,
    fallback: &'a dyn ExpressionParseEnvironment,
}

impl<'a> ExpressionParseEnvironment for FallbackExpressionParseEnvironemnt<'a> {
    fn resolve_procedure_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        self.main.resolve_procedure_identifier(ident)
            .or(self.fallback.resolve_procedure_identifier(ident))
    }

    fn resolve_struct_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        self.main.resolve_struct_identifier(ident)
            .or(self.fallback.resolve_struct_identifier(ident))
    }

    fn resolve_variable_ident(&self, ident: &dyn AsRef<str>) -> Result<usize> {
        self.main.resolve_variable_ident(ident)
            .or(self.fallback.resolve_variable_ident(ident))
    }
}

impl<'a> FallbackExpressionParseEnvironemnt<'a> {
    pub fn new(main: &'a dyn ExpressionParseEnvironment, fallback: &'a dyn ExpressionParseEnvironment) -> Self {
        Self { main, fallback }
    }
}

#[derive(Debug)]
pub(crate) struct UsingExpressionParseEnvironment {
    entries: Vec<UsingEntry>,
}

#[derive(Debug)]
struct UsingEntry {
    module_name: String,
    member_name: Option<String>,
}

impl ExpressionParseEnvironment for UsingExpressionParseEnvironment {
    fn resolve_procedure_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        self.resolve(ident)
    }

    fn resolve_struct_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        self.resolve(ident)
    }

    fn resolve_variable_ident(&self, ident: &dyn AsRef<str>) -> Result<usize> {
        Err(CompilerError::NoSuchVariable { ident: ident.as_ref().to_string() }.boxed())
    }
}

impl UsingExpressionParseEnvironment {
    pub(crate) fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub(crate) fn push(&mut self, module_name: String, member_name: Option<String>) {
        self.entries.push(UsingEntry { module_name, member_name });
    }

    fn resolve(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        for entry in &self.entries {
            if entry.member_name.as_ref().is_none_or(|member_name| ident.as_ref() == member_name as &str) {
                return Ok(ModuleAddress::new(entry.module_name.clone(), ident.as_ref().into()));
            }
        }

        Err(CompilerError::NoSuchMember { member_identifier: ident.as_ref().to_string() }.boxed())
    }
}

pub struct Compiler {
    state: Box<dyn CompilerState>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn finalize(self) -> Result<Module> {
        self.state.finalize()
    }

    pub fn compile(mut self, tokens: impl Iterator<Item = ContextualizedToken>, environment: &mut CompilerEnvironment) -> Result<Module> {
        for token in tokens {
            let line = token.line_index + 1;
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

#[derive(Debug)]
pub struct CompilerEnvironment {
    procedure_ident_map: HashMap<String, ModuleAddress>,
    struct_ident_map: HashMap<String, ModuleAddress>,

    file_read_queue: Vec<ImportAddress>,
    read_modules: HashSet<ImportAddress>,
}

impl Default for CompilerEnvironment {
    fn default() -> Self {
        Self::new()
    }
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
    fn resolve_procedure_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        self.procedure_ident_map
            .get(ident.as_ref())
            .ok_or(
                CompilerError::InvalidExpression {
                    message: format!(
                        "Single identifier '{}' could not be mapped to a procedure!", ident.as_ref()
                    ),
                }
                .boxed(),
            )
            .map(|address| address.to_owned())
    }

    fn resolve_struct_identifier(&self, ident: &dyn AsRef<str>) -> Result<ModuleAddress> {
        self.struct_ident_map
            .get(ident.as_ref())
            .ok_or(
                CompilerError::InvalidExpression {
                    message: format!(
                        "Single identifier '{}' could not be mapped to a struct!", ident.as_ref()
                    ),
                }
                .boxed(),
            )
            .map(|address| address.to_owned())
    }
    
    fn resolve_variable_ident(&self, ident: &dyn AsRef<str>) -> Result<usize> {
        Err(
            CompilerError::NoSuchVariable { ident: ident.as_ref().to_string() }.boxed()
        )
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
use otr_ffi::external::ExternalModule;