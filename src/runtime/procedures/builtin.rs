use std::rc::Rc;

use crate::runtime::{procedures::Procedure, Environment, Expression, RuntimeError, Value};

#[derive(Debug)]
pub struct SizeProcedure;

impl Procedure for SizeProcedure {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let arg = arguments.first().ok_or(RuntimeError {
            message: "Missing argument!".into(),
        })?;

        match arg {
            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
            Value::Struct(s) => Ok(Value::Integer(s.get_members().len() as i64)),
            other => Err(RuntimeError {
                message: format!("Cannot identify size of {}!", other.get_type_id()),
            }),
        }
    }
}
