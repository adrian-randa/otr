use crate::runtime::{RuntimeError, Value, module::Module, procedures::Procedure};


pub(crate) fn get_module() -> Module {
    let mut module = Module::default();

    module.insert_procedure("length".into(), Box::new(StringLengthProcdure), true);
    module.insert_procedure("toCharArray".into(), Box::new(StringToCharArrayProcedure), true);
    module.insert_procedure("split".into(), Box::new(StringSplitProcedure), true);
    
    module
}

#[derive(Debug)]
pub(crate) struct StringLengthProcdure;

impl Procedure for StringLengthProcdure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<crate::runtime::Value>) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
        let str = arguments.get(0).ok_or(RuntimeError {
            message: "Missing argument for 'Strings::length'!".into()
        })?;

        match str {
            Value::String(str) => {
                Ok(Value::Integer(str.len() as i64))
            }

            other => {Err(RuntimeError {
                message: format!("Cannot compute string length for value of type '{}'", other.get_type_id())
            })}
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringToCharArrayProcedure;

impl Procedure for StringToCharArrayProcedure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let str = arguments.get(0).ok_or(RuntimeError {
            message: "Missing argument for 'Strings::toCharArray'!".into()
        })?;

        match str {
            Value::String(str) => {
                Ok(Value::Array(str.chars().map(|c| Value::Char(c)).collect()))
            }

            other => {Err(RuntimeError {
                message: format!("Cannot compute Char array from value of type '{}'", other.get_type_id())
            })}
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringSplitProcedure;

impl Procedure for StringSplitProcedure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let str = arguments.get(0).ok_or(RuntimeError {
            message: "Missing string argument for 'Strings::toCharArray'!".into()
        })?;
        let str = if let Value::String(str) = str { str } else {
            return Err(RuntimeError {
                message: format!("Cannot split value of type '{}'!", str.get_type_id())
            });
        };

        let pattern = arguments.get(1).ok_or(RuntimeError {
            message: "Missing pattern argument for 'Strings::toCharArray'!".into()
        })?;
        let pattern = if let Value::String(pattern) = pattern { pattern } else {
            return Err(RuntimeError {
                message: format!("Cannot split value of type '{}'!", pattern.get_type_id())
            });
        };

        Ok(Value::Array(str.split(pattern).map(|part| Value::String(part.into())).collect()))
    }
}