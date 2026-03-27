use num::ToPrimitive;

use crate::runtime::environment::Environment;
use crate::core::r#type::Type;
use crate::runtime::environment::features::FeatureBuilder;
use crate::runtime::module::{Module, RuntimeModule};
use crate::runtime::procedures::RuntimeProcedure;
use crate::runtime::{procedures::Procedure, RuntimeError, Value};

use crate::error::Result;

pub(crate) struct StringsFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl StringsFeatureBuilder {
    pub(crate) fn new() -> Box<dyn FeatureBuilder> {
        Box::new(Self { })
    }
}

impl FeatureBuilder for StringsFeatureBuilder {
    fn add_arg(&mut self, _arg_ident: &dyn AsRef<str>, _arg_value: &dyn AsRef<str>) -> Result<()> {
        Err(RuntimeError::Unknown { message: "Feature arguments not supported!".into() }.boxed())
    }

    fn build(&mut self) -> Result<RuntimeModule<'static>> {
        Ok(RuntimeModule::Abstract(Box::new(StringsFeature)))
    }
}

#[derive(Debug)]
struct StringsFeature;

impl Module for StringsFeature {
    fn get_procedure(
        &'_ self,
        identifier: &String,
        _private_access: bool,
    ) -> Result<crate::runtime::procedures::RuntimeProcedure<'_>> {
        match identifier as &str {
            "length" => Ok(RuntimeProcedure::AbstractRef(&StringLengthProcdure)),
            "split" => Ok(RuntimeProcedure::AbstractRef(&StringSplitProcedure)),
            "toString" => Ok(RuntimeProcedure::AbstractRef(&ToStringProcedure)),
            "toCharArray" => Ok(RuntimeProcedure::AbstractRef(&StringToCharArrayProcedure)),
            "fromBytes" => Ok(RuntimeProcedure::AbstractRef(&FromBytesProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &'_ self,
        _struct_identifier: &String,
        _procedure_identifier: &String,
        _private_access: bool,
    ) -> Result<crate::runtime::procedures::RuntimeProcedure<'_>> {
        todo!()
    }

    fn get_struct(&self, _identifier: &String, _private_access: bool) -> Result<crate::core::r#struct::Struct> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct StringLengthProcdure;

impl Procedure for StringLengthProcdure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
        arguments: Vec<crate::runtime::Value>,
    ) -> Result<crate::runtime::Value> {
        let str = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "string".into(),
            }
            .boxed(),
        )?;

        match str {
            Value::String(str) => Ok(Value::Integer(str.len() as i64)),

            other => Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: other.get_type_id(),
            }
            .boxed()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringToCharArrayProcedure;

impl Procedure for StringToCharArrayProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let str = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "string".into(),
            }
            .boxed(),
        )?;

        match str {
            Value::String(str) => Ok(Value::Array(str.chars().map(|c| Value::Char(c)).collect())),

            other => Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: other.get_type_id(),
            }
            .boxed()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringSplitProcedure;

impl Procedure for StringSplitProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let str = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "string".into(),
            }
            .boxed(),
        )?;
        let str = if let Value::String(str) = str {
            str
        } else {
            return Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: str.get_type_id(),
            }
            .boxed());
        };

        let pattern = arguments.get(1).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "pattern".into(),
            }
            .boxed(),
        )?;
        let pattern = if let Value::String(pattern) = pattern {
            pattern
        } else {
            return Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: pattern.get_type_id(),
            }
            .boxed());
        };

        Ok(Value::Array(
            str.split(pattern)
                .map(|part| Value::String(part.into()))
                .collect(),
        ))
    }
}

#[derive(Debug)]
pub struct ToStringProcedure;

impl Procedure for ToStringProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let value = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "value".into(),
            }
            .boxed(),
        )?;

        Ok(Value::String(value.to_string()))
    }
}

#[derive(Debug)]
pub(crate) struct FromBytesProcedure;

impl Procedure for FromBytesProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let value = arguments.get(0).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "value".into(),
            }
            .boxed(),
        )?;

        let bytes = if let Value::Array(arr) = value {
            let mut bytes = Vec::with_capacity(arr.len());
            for (index, item) in arr.into_iter().enumerate() {
                if let Value::Integer(byte) = item {
                    bytes.push(
                        byte.to_u8().ok_or(
                            RuntimeError::Unknown {
                                message: format!(
                                    "Element of array at index {index} is not a valid byte!"
                                ),
                            }
                            .boxed(),
                        )?,
                    );
                };
            }
            bytes
        } else {
            return Err(RuntimeError::TypeMismatch {
                expected: Type::Array,
                found: value.get_type_id(),
            }
            .boxed());
        };

        Ok(Value::String(String::from_utf8(bytes).map_err(|err| {
            RuntimeError::Unknown {
                message: err.to_string(),
            }
            .boxed()
        })?))
    }
}
