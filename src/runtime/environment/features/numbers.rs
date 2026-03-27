use crate::runtime::environment::features::FeatureBuilder;
use crate::runtime::module::{Module, RuntimeModule};
use crate::runtime::procedures::RuntimeProcedure;
use crate::runtime::{procedures::Procedure, RuntimeError, Value};

use crate::error::Result;

pub(crate) struct NumbersFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl NumbersFeatureBuilder {
    pub(crate) fn new() -> Box<dyn FeatureBuilder> {
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
        identifier: &String,
        _private_access: bool,
    ) -> Result<crate::runtime::procedures::RuntimeProcedure<'_>> {
        match identifier as &str {
            "parse" => Ok(RuntimeProcedure::AbstractRef(&NumberParseProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &String,
        procedure_identifier: &String,
        _private_access: bool,
    ) -> Result<crate::runtime::procedures::RuntimeProcedure<'_>> {
        Err(RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: procedure_identifier.to_string(), struct_identifier: struct_identifier.to_string() }.boxed())
    }

    fn get_struct(&self, identifier: &String, _private_access: bool) -> Result<crate::core::r#struct::Struct> {
        Err(RuntimeError::StructNotDefined { struct_identifier: identifier.to_string() }.boxed())
    }
}

#[derive(Debug)]
pub(crate) struct NumberParseProcedure;

impl Procedure for NumberParseProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
        arguments: Vec<Value>,
    ) -> Result<crate::runtime::Value> {
        let value = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "number".into(),
            }
            .boxed(),
        )?;

        match value {
            Value::Char(c) => {
                let n = *c as u8;

                if n < '0' as u8 || n > '9' as u8 {
                    Err(RuntimeError::Unknown {
                        message: format!("'{}' is not a valid digit!", c),
                    }
                    .boxed())
                } else {
                    Ok(Value::Integer((n - '0' as u8) as i64))
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
                expected: crate::core::r#type::Type::String,
                found: other.get_type_id(),
            }
            .boxed()),
        }
    }
}
