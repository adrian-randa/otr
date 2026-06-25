use crate::{RuntimeError, Value, environment::{Environment, features::FeatureBuilder}, module::{Module, RuntimeModule}, procedures::{Procedure, RuntimeProcedure}};

use otr_core::{error::Result, r#struct::Struct, r#type::Type};

pub(crate) struct NumbersFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl NumbersFeatureBuilder {
    pub(crate) fn new_boxed() -> Box<dyn FeatureBuilder> {
        Box::new(Self { })
    }
}

impl FeatureBuilder for NumbersFeatureBuilder {
    fn add_arg(&mut self, _arg_ident: &dyn AsRef<str>, _arg_value: &dyn AsRef<str>) -> Result<()> {
        Err(RuntimeError::Unknown { message: "Feature arguments not supported!".into() }.boxed())
    }

    fn build(&mut self) -> Result<RuntimeModule<'static>> {
        Ok(RuntimeModule::Abstract(Box::new(NumbersFeature)))
    }
}

#[derive(Debug)]
struct NumbersFeature;

impl Module for NumbersFeature {
    fn get_procedure(
        &'_ self,
        identifier: &str,
        _private_access: bool,
    ) -> Result<RuntimeProcedure<'_>> {
        match identifier as &str {
            "parse" => Ok(RuntimeProcedure::AbstractRef(&NumberParseProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &str,
        procedure_identifier: &str,
        _private_access: bool,
    ) -> Result<RuntimeProcedure<'_>> {
        Err(RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: procedure_identifier.to_string(), struct_identifier: struct_identifier.to_string() }.boxed())
    }

    fn get_struct(&self, identifier: &str, _private_access: bool) -> Result<Struct> {
        Err(RuntimeError::StructNotDefined { struct_identifier: identifier.to_string() }.boxed())
    }
}

#[derive(Debug)]
pub(crate) struct NumberParseProcedure;

impl Procedure for NumberParseProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let value = arguments.first().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "number".into(),
            }
            .boxed(),
        )?;

        match value {
            Value::Char(c) => {
                let n = *c as u8;

                if !n.is_ascii_digit() {
                    Err(RuntimeError::Unknown {
                        message: format!("'{}' is not a valid digit!", c),
                    }
                    .boxed())
                } else {
                    Ok(Value::Integer((n - b'0') as i64))
                }
            }
            Value::String(str) => {
                if let Ok(integer) = str.parse() {
                    Ok(Value::Integer(integer))
                } else if let Ok(float) = str.parse() {
                    Ok(Value::Float(float))
                } else {
                    Err(RuntimeError::Unknown {
                        message: format!("'{}' is not a valid number!", str),
                    }
                    .boxed())
                }
            }

            other => Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: other.get_type_id(),
            }
            .boxed()),
        }
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}
