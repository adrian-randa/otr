use crate::{RuntimeError, Value, environment::{Environment, features::FeatureBuilder}, module::{Module, RuntimeModule}, procedures::{Procedure, RuntimeProcedure}};

use otr_core::error::Result;

pub(crate) struct DebugFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl DebugFeatureBuilder {
    pub(crate) fn new() -> Box<dyn FeatureBuilder> {
        Box::new(Self { })
    }
}

impl FeatureBuilder for DebugFeatureBuilder {
    fn add_arg(&mut self, _arg_ident: &dyn AsRef<str>, _arg_value: &dyn AsRef<str>) -> Result<()> {
        Err(RuntimeError::Unknown { message: "Feature arguments not supported!".into() }.boxed())
    }

    fn build(&mut self) -> Result<RuntimeModule<'static>> {
        Ok(RuntimeModule::Abstract(Box::new(DebugFeature)))
    }
}


#[derive(Debug)]
struct DebugFeature;

impl Module for DebugFeature {
    fn get_procedure(
        &'_ self,
        identifier: &String,
        _private_access: bool,
    ) -> Result<crate::procedures::RuntimeProcedure<'_>> {
        match identifier as &str {
            "print" => Ok(RuntimeProcedure::AbstractRef(&DebugPrintProcedure)),
            "println" => Ok(RuntimeProcedure::AbstractRef(&DebugPrintlnProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &String,
        procedure_identifier: &String,
        _private_access: bool,
    ) -> Result<crate::procedures::RuntimeProcedure<'_>> {
        Err(RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: procedure_identifier.to_string(), struct_identifier: struct_identifier.to_string() }.boxed())
    }

    fn get_struct(&self, identifier: &String, _private_access: bool) -> Result<otr_core::r#struct::Struct> {
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
        _environment: crate::environment::Environment,
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
