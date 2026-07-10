use crate::{RuntimeError, Value, environment::{Environment, features::FeatureBuilder}, module::{Module, RuntimeModule}, procedures::{Procedure, RuntimeProcedure}};

use otr_core::error::Result;

pub(crate) struct MathFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl MathFeatureBuilder {
    pub(crate) fn new_boxed() -> Box<dyn FeatureBuilder> {
        Box::new(Self { })
    }
}

impl FeatureBuilder for MathFeatureBuilder {
    fn add_arg(&mut self, _arg_ident: &dyn AsRef<str>, _arg_value: &dyn AsRef<str>) -> Result<()> {
        Err(RuntimeError::Unknown { message: "Feature arguments not supported!".into() }.boxed())
    }

    fn build(&mut self) -> Result<RuntimeModule<'static>> {
        Ok(RuntimeModule::Abstract(Box::new(MathFeature)))
    }
}


#[derive(Debug)]
struct MathFeature;

impl Module for MathFeature {
    fn get_procedure(
        &'_ self,
        identifier: &str,
        _private_access: bool,
    ) -> Result<RuntimeProcedure<'_>> {
        match identifier as &str {
            "sin" => Ok(RuntimeProcedure::AbstractRef(&SinProcedure)),
            "cos" => Ok(RuntimeProcedure::AbstractRef(&CosProcedure)),
            "tan" => Ok(RuntimeProcedure::AbstractRef(&TanProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &str,
        procedure_identifier: &str,
        _private_access: bool,
    ) -> Result<crate::procedures::RuntimeProcedure<'_>> {
        Err(RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: procedure_identifier.to_string(), struct_identifier: struct_identifier.to_string() }.boxed())
    }

    fn get_struct(&self, identifier: &str, _private_access: bool) -> Result<otr_core::r#struct::Struct> {
        Err(RuntimeError::StructNotDefined { struct_identifier: identifier.to_string() }.boxed())
    }

    fn get_operation(&self, struct_identifier: &str, operator: otr_core::expression::Operator, _private_access: bool) -> Result<RuntimeProcedure<'_>> {
        Err(RuntimeError::OperatorNotOverloaded { struct_identifier: struct_identifier.to_string(), operator }.boxed())
    }
}


#[derive(Debug)]
pub(crate) struct SinProcedure;

impl Procedure for SinProcedure {
    fn call(
        &self,
        _environment: Environment,
        mut arguments: Vec<Value>,
    ) -> Result<Value> {
        let arg = arguments.pop().ok_or(
            RuntimeError::NoSuchVariable { variable_identifier: "x".into() }.boxed()
        )?;

        Ok(Value::Float(
            match arg {
                Value::Integer(x) => (x as f64).sin(),
                Value::Float(x) => x.sin(),
                _ => return Err(RuntimeError::Unknown {
                    message: "Argument must only be of type Integer or Float".into()
                }.boxed())
            }
        ))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct CosProcedure;

impl Procedure for CosProcedure {
    fn call(
        &self,
        _environment: Environment,
        mut arguments: Vec<Value>,
    ) -> Result<Value> {
        let arg = arguments.pop().ok_or(
            RuntimeError::NoSuchVariable { variable_identifier: "x".into() }.boxed()
        )?;

        Ok(Value::Float(
            match arg {
                Value::Integer(x) => (x as f64).cos(),
                Value::Float(x) => x.cos(),
                _ => return Err(RuntimeError::Unknown {
                    message: "Argument must only be of type Integer or Float".into()
                }.boxed())
            }
        ))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct TanProcedure;

impl Procedure for TanProcedure {
    fn call(
        &self,
        _environment: Environment,
        mut arguments: Vec<Value>,
    ) -> Result<Value> {
        let arg = arguments.pop().ok_or(
            RuntimeError::NoSuchVariable { variable_identifier: "x".into() }.boxed()
        )?;

        Ok(Value::Float(
            match arg {
                Value::Integer(x) => (x as f64).tan(),
                Value::Float(x) => x.tan(),
                _ => return Err(RuntimeError::Unknown {
                    message: "Argument must only be of type Integer or Float".into()
                }.boxed())
            }
        ))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}