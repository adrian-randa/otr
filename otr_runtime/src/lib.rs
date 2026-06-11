use std::marker::PhantomData;
use std::rc::Rc;

use otr_core::CompiledObject;
use otr_core::expression::{Expression, ProcedureCallExpression};
use otr_core::module::{CompiledModule, ModuleAddress};
use otr_core::value::Value;
use crate::error::context::{HintContextDecorator};
use crate::environment::Environment;

use otr_core::error::Result;
use crate::error::RuntimeError;
use crate::expressions::eval_expression;
use crate::module::RuntimeModule;

pub mod environment;
mod expressions;
mod module;
mod procedures;
mod value;
mod error;

#[derive(Debug)]
pub struct RuntimeObject<'a> {
    pub(crate) base_environement: Environment<'a>,
    pub(crate) entrypoint: Option<ModuleAddress>,
}

impl From<CompiledObject> for RuntimeObject<'_> {
    fn from(mut object: CompiledObject) -> Self {
        let mut runtime_object = Self {
            base_environement: Environment::default(),
            entrypoint: object.root().map(|root| (&root as &str, "main").into())
        };

        for (identifier, module) in object {
            runtime_object.base_environement.load_module(
                identifier,
                Rc::new(RuntimeModule::Compiled(module))
            );
        }

        runtime_object    
    }
}

pub struct RuntimeObjectBuilderNoRoot;
pub struct RuntimeObjectBuilderWithRoot;
pub struct RuntimeObjectBuilder<T> {
    runtime_object: RuntimeObject<'static>,
    phantom_data: PhantomData<T>,
}

impl RuntimeObjectBuilder<RuntimeObjectBuilderNoRoot> {
    pub fn with_root(mut self, root_module: CompiledModule, module_ident: String) -> RuntimeObjectBuilder<RuntimeObjectBuilderWithRoot> {
        self.runtime_object.base_environement.load_module(module_ident.clone(), Rc::new(RuntimeModule::Compiled(root_module)));
        self.runtime_object.entrypoint = Some(ModuleAddress::new(module_ident, "main".into()));

        RuntimeObjectBuilder {
            runtime_object: self.runtime_object,
            phantom_data: PhantomData
        }
    }
}

impl RuntimeObjectBuilder<RuntimeObjectBuilderWithRoot> {
    pub fn with_module(mut self, module: CompiledModule, module_ident: String) -> Self {
        self.runtime_object.base_environement.load_module(module_ident, Rc::new(RuntimeModule::Compiled(module)));
        self
    }

    pub fn build(self) -> RuntimeObject<'static> {
        self.runtime_object
    }
}

impl RuntimeObject<'_> {
    fn new() -> Self {
        Self {
            base_environement: Environment::new("".into(), 0),
            entrypoint: None,
        }
    }

    pub fn builder(base_environment: Environment<'static>) -> RuntimeObjectBuilder<RuntimeObjectBuilderNoRoot> {
        let mut runtime_object = RuntimeObject::new();
        runtime_object.base_environement = base_environment;
        
        RuntimeObjectBuilder {
            runtime_object: runtime_object,
            phantom_data: PhantomData,
        }
    }

    pub fn execute(self) -> Result<Value> {
        let entrypoint = self.entrypoint.ok_or(
            HintContextDecorator {
                error: RuntimeError::NoEntrypoint.boxed(),
                message: "If you want to run the specified file as a script, please annotate a procedure as the entrypoint.".into()
            }.boxed()
        )?;

        let main_expression = Expression::ProcedureCall(
            ProcedureCallExpression::new(entrypoint, Vec::new())
        );

        eval_expression(&main_expression, &self.base_environement)
    }
}

mod scope;

pub use crate::environment::environment_builder::{self, *};