use crate::runtime::{RuntimeError, Value, environment::Environment, module::Module, procedures::Procedure};

use crate::error::Result;

pub(crate) fn get_module() -> Module {
    let mut module = Module::default();

    module.insert_procedure("new".into(), Box::new(NewArrayProcedure), true);
    module.insert_procedure("size".into(), Box::new(ArraySizeProcedure), true);

    module
}


#[derive(Debug)]
pub(crate) struct NewArrayProcedure;

impl Procedure for NewArrayProcedure {
    fn call(&self, _environment: Environment, arguments: Vec<Value>) -> Result<Value> {
        let size = arguments.get(0).or(Some(&Value::Integer(0))).unwrap();

        if let Value::Integer(size) = size {
            Ok(Value::Array(vec![Value::Null; *size as usize]))
        } else {
            Err(RuntimeError::TypeMismatch { expected: crate::runtime::Type::Integer, found: size.get_type_id() }.boxed())
        }
    }
}

#[derive(Debug)]
pub(crate) struct ArraySizeProcedure;

impl Procedure for ArraySizeProcedure {
    fn call(&self, _environment: Environment, arguments: Vec<Value>) -> Result<Value> {
        let arg = arguments.first().ok_or(RuntimeError::NoSuchVariable { variable_identifier: "array".into() }.boxed())?;

        match arg {
            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
            other => Err(RuntimeError::TypeMismatch { expected: crate::runtime::Type::Array, found: other.get_type_id() }.boxed()),
        }
    }
}