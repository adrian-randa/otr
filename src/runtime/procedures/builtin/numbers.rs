use crate::runtime::{RuntimeError, Value, module::Module, procedures::Procedure};

use crate::error::Result;

pub(crate) fn get_module() -> Module {
    let mut module = Module::default();

    module.insert_procedure("parse".into(), Box::new(NumberParseProcedure), true);
    
    module
}

#[derive(Debug)]
pub(crate) struct NumberParseProcedure;

impl Procedure for NumberParseProcedure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<crate::runtime::Value>) -> Result<crate::runtime::Value> {
        let value = arguments.get(0).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "number".into() }.boxed())?;

        match value {

            Value::Char(c) => {
                let n = *c as u8;

                if n < '0' as u8 || n > '9' as u8 {
                    Err(RuntimeError::Unknown {
                        message: format!("'{}' is not a valid digit!", c)
                    }.boxed())
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
                        message: format!("'{}' is not a valid number!", str)
                    }.boxed())
                }
            }

            other => Err(RuntimeError::TypeMismatch { expected: crate::runtime::Type::String, found: other.get_type_id() }.boxed())
        }
    }
}