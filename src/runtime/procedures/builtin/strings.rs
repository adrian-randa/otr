use crate::runtime::{RuntimeError, Value, module::Module, procedures::Procedure};

use crate::error::Result;

pub(crate) fn get_module() -> Module {
    let mut module = Module::default();

    module.insert_procedure("length".into(), Box::new(StringLengthProcdure), true);
    module.insert_procedure("toCharArray".into(), Box::new(StringToCharArrayProcedure), true);
    module.insert_procedure("split".into(), Box::new(StringSplitProcedure), true);
    module.insert_procedure("toString".into(), Box::new(ToStringProcedure), true);
    
    module
}

#[derive(Debug)]
pub(crate) struct StringLengthProcdure;

impl Procedure for StringLengthProcdure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<crate::runtime::Value>) -> Result<crate::runtime::Value> {
        let str = arguments.get(0).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "string".into() }.boxed())?;

        match str {
            Value::String(str) => {
                Ok(Value::Integer(str.len() as i64))
            }

            other => {Err(RuntimeError::TypeMismatch { expected: crate::runtime::Type::String, found: other.get_type_id() }.boxed())}
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringToCharArrayProcedure;

impl Procedure for StringToCharArrayProcedure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<Value>) -> Result<Value> {
        let str = arguments.get(0).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "string".into() }.boxed())?;

        match str {
            Value::String(str) => {
                Ok(Value::Array(str.chars().map(|c| Value::Char(c)).collect()))
            }

            other => {Err(RuntimeError::TypeMismatch { expected: crate::runtime::Type::String, found: other.get_type_id() }.boxed())}
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringSplitProcedure;

impl Procedure for StringSplitProcedure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<Value>) -> Result<Value> {
        let str = arguments.get(0).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "string".into() }.boxed())?;
        let str = if let Value::String(str) = str { str } else {
            return Err(RuntimeError::TypeMismatch { expected: crate::runtime::Type::String, found: str.get_type_id() }.boxed());
        };

        let pattern = arguments.get(1).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "pattern".into() }.boxed())?;
        let pattern = if let Value::String(pattern) = pattern { pattern } else {
            return Err(RuntimeError::TypeMismatch { expected: crate::runtime::Type::String, found: pattern.get_type_id() }.boxed());
        };

        Ok(Value::Array(str.split(pattern).map(|part| Value::String(part.into())).collect()))
    }
}

#[derive(Debug)]
pub struct ToStringProcedure;

impl Procedure for ToStringProcedure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<Value>) -> Result<Value> {
        let value = arguments.get(0).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "value".into() }.boxed())?;

        Ok(Value::String(value.to_string()))
    }
}