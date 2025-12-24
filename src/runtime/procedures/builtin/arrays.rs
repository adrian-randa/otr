use crate::runtime::{RuntimeError, Value, environment::Environment, module::Module, procedures::Procedure};

pub fn get_module() -> Module {
    let mut module = Module::default();

    module.insert_procedure("new".into(), Box::new(NewArrayProcedure), true);
    module.insert_procedure("size".into(), Box::new(ArraySizeProcedure), true);

    module
}


#[derive(Debug)]
pub struct NewArrayProcedure;

impl Procedure for NewArrayProcedure {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
        let size = arguments.get(0).or(Some(&Value::Integer(0))).unwrap();

        if let Value::Integer(size) = size {
            Ok(Value::Array(vec![Value::Null; *size as usize]))
        } else {
            Err(RuntimeError {
                message: format!("Array size needs to be of type Integer, found {}!", size.get_type_id())
            })
        }
    }
}

#[derive(Debug)]
pub struct ArraySizeProcedure;

impl Procedure for ArraySizeProcedure {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let arg = arguments.first().ok_or(RuntimeError {
            message: "Missing argument!".into(),
        })?;

        match arg {
            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
            other => Err(RuntimeError {
                message: format!("Cannot identify size of {}!", other.get_type_id()),
            }),
        }
    }
}