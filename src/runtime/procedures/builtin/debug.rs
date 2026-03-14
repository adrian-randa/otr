use crate::runtime::environment::Environment;
use crate::runtime::module::{Module, RuntimeModule};
use crate::runtime::procedures::RuntimeProcedure;
use crate::runtime::{procedures::Procedure, RuntimeError, Value};

use crate::error::Result;

pub(crate) fn get_module() -> RuntimeModule<'static> {
    RuntimeModule::Abstract(Box::new(DebugModule))
}

#[derive(Debug)]
struct DebugModule;

impl Module for DebugModule {
    fn get_procedure(
        &self,
        identifier: &String,
        _private_access: bool,
    ) -> Result<crate::runtime::procedures::RuntimeProcedure> {
        match identifier as &str {
            "print" => Ok(RuntimeProcedure::AbstractRef(&DebugPrintProcedure)),
            "println" => Ok(RuntimeProcedure::AbstractRef(&DebugPrintlnProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &self,
        struct_identifier: &String,
        procedure_identifier: &String,
        _private_access: bool,
    ) -> Result<crate::runtime::procedures::RuntimeProcedure> {
        Err(RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: procedure_identifier.to_string(), struct_identifier: struct_identifier.to_string() }.boxed())
    }

    fn get_struct(&self, identifier: &String, _private_access: bool) -> Result<crate::core::r#struct::Struct> {
        Err(RuntimeError::StructNotDefined { struct_identifier: identifier.to_string() }.boxed())
    }
}


#[derive(Debug)]
pub(crate) struct DebugPrintProcedure;

impl Procedure for DebugPrintProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let arg = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "content".into(),
            }
            .boxed(),
        )?;

        print!("{}", arg.to_string());

        Ok(Value::Null)
    }
}

#[derive(Debug)]
pub(crate) struct DebugPrintlnProcedure;

impl Procedure for DebugPrintlnProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let arg = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "content".into(),
            }
            .boxed(),
        )?;

        println!("{}", arg.to_string());

        Ok(Value::Null)
    }
}
