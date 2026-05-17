use std::cmp::Ordering;

use crate::{RuntimeError, Value, environment::{Environment, features::FeatureBuilder}, module::{Module, RuntimeModule}, procedures::{Procedure, RuntimeProcedure}};

use otr_core::Result;

pub(crate) struct ArraysFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl ArraysFeatureBuilder {
    pub(crate) fn new() -> Box<dyn FeatureBuilder> {
        Box::new(Self { })
    }
}

impl FeatureBuilder for ArraysFeatureBuilder {
    fn add_arg(&mut self, _arg_ident: &dyn AsRef<str>, _arg_value: &dyn AsRef<str>) -> Result<()> {
        Err(RuntimeError::Unknown { message: "Feature arguments not supported!".into() }.boxed())
    }

    fn build(&mut self) -> Result<RuntimeModule<'static>> {
        Ok(RuntimeModule::Abstract(Box::new(ArraysFeature)))
    }
}

#[derive(Debug)]
struct ArraysFeature;

impl Module for ArraysFeature {
    fn get_procedure(
        &'_ self,
        identifier: &String,
        _private_access: bool,
    ) -> Result<crate::procedures::RuntimeProcedure<'_>> {
        match identifier as &str {
            "new" => Ok(RuntimeProcedure::AbstractRef(&NewArrayProcedure)),
            "size" => Ok(RuntimeProcedure::AbstractRef(&ArraySizeProcedure)),
            "sort" => Ok(RuntimeProcedure::AbstractRef(&ArraySortProcedure)),

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
pub(crate) struct NewArrayProcedure;

impl Procedure for NewArrayProcedure {
    fn call(&self, _environment: Environment, arguments: Vec<Value>) -> Result<Value> {
        let size = arguments.get(0).or(Some(&Value::Integer(0))).unwrap();

        if let Value::Integer(size) = size {
            Ok(Value::Array(vec![Value::Null; *size as usize]))
        } else {
            Err(RuntimeError::TypeMismatch {
                expected: otr_core::r#type::Type::Integer,
                found: size.get_type_id(),
            }
            .boxed())
        }
    }
}

#[derive(Debug)]
pub(crate) struct ArraySizeProcedure;

impl Procedure for ArraySizeProcedure {
    fn call(&self, _environment: Environment, arguments: Vec<Value>) -> Result<Value> {
        let arg = arguments.first().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "array".into(),
            }
            .boxed(),
        )?;

        match arg {
            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
            other => Err(RuntimeError::TypeMismatch {
                expected: otr_core::r#type::Type::Array,
                found: other.get_type_id(),
            }
            .boxed()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ArraySortProcedure;

impl Procedure for ArraySortProcedure {
    fn call(&self, _environment: Environment, mut arguments: Vec<Value>) -> Result<Value> {
        let arg = arguments.pop().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "array".into(),
            }
            .boxed(),
        )?;

        let mut arr = match arg {
            Value::Array(arr) => arr,
            other => return Err(RuntimeError::TypeMismatch {
                expected: otr_core::r#type::Type::Array,
                found: other.get_type_id(),
            }
            .boxed()),
        };

        arr.sort_by(
            |l, r| crate::value::compare(l, r).unwrap_or(Ordering::Equal)
        );

        Ok(Value::Array(arr))
    }
}
