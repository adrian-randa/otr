use crate::runtime::{RuntimeError, Value, module::Module, procedures::Procedure};

pub(crate) fn get_module() -> Module {
    let mut module = Module::default();

    module.insert_procedure("parse".into(), Box::new(NumberParseProcedure), true);
    
    module
}

#[derive(Debug)]
pub(crate) struct NumberParseProcedure;

impl Procedure for NumberParseProcedure {
    fn call(&self, _environment: crate::runtime::environment::Environment, arguments: Vec<crate::runtime::Value>) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
        let value = arguments.get(0).ok_or(RuntimeError {
            message: "Missing argument for 'Numbers::parse'!".into()
        })?;

        match value {

            Value::Char(c) => {
                let n = *c as u8;

                if n < '0' as u8 || n > '9' as u8 {
                    Err(RuntimeError {
                        message: format!("'{}' is not a valid digit!", c)
                    })
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
                    Err(RuntimeError {
                        message: format!("'{}' is not a valid number!", str)
                    })
                }
            }

            other => Err(RuntimeError {
                message: format!("Cannot parse number from value of type {}!", other.get_type_id())
            })
        }
    }
}