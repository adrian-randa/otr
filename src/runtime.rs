use std::rc::Rc;

use crate::core::CompiledObject;
use crate::core::expression::{Expression, ProcedureCallExpression};
use crate::core::module::ModuleAddress;
use crate::core::value::Value;
use crate::error::context::{HintContextDecorator};
use crate::error::runtime_error::RuntimeError;
use crate::runtime::environment::Environment;

use crate::error::Result;
use crate::runtime::expressions::eval_expression;
use crate::runtime::module::RuntimeModule;

pub mod environment;
pub mod expressions;
pub mod module;
pub mod procedures;

#[derive(Debug)]
pub struct RuntimeObject<'a> {
    pub(crate) base_environement: Environment<'a>,
    pub(crate) entrypoint: Option<ModuleAddress>,
}

impl From<CompiledObject> for RuntimeObject<'_> {
    fn from(mut object: CompiledObject) -> Self {
        let mut runtime_object = Self {
            base_environement: Environment::default(),
            entrypoint: object.entrypoint()
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

impl RuntimeObject<'_> {
    pub(crate) fn _new() -> Self {
        Self {
            base_environement: Environment::new("".into()),
            entrypoint: None,
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

pub mod scope;
