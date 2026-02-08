use crate::runtime::{RuntimeError, Value, module::Module, procedures::Procedure};

use crate::error::Result;

pub(crate) fn get_module() -> Module {
    let mut module = Module::default();

    module.insert_procedure("print".into(), Box::new(DebugPrintProcedure), true);
    module.insert_procedure("println".into(), Box::new(DebugPrintlnProcedure), true);
    
    module
}

#[derive(Debug)]
pub(crate) struct DebugPrintProcedure;

impl Procedure for DebugPrintProcedure {
    fn call(&self, environment: crate::runtime::environment::Environment, arguments: Vec<Value>) -> Result<Value> {
        let arg = arguments.get(0).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "content".into() }.boxed())?;

        print!("{}", arg.to_string());

        Ok(Value::Null)
    }
}

#[derive(Debug)]
pub(crate) struct DebugPrintlnProcedure;

impl Procedure for DebugPrintlnProcedure {
    fn call(&self, environment: crate::runtime::environment::Environment, arguments: Vec<Value>) -> Result<Value> {
        let arg = arguments.get(0).ok_or(RuntimeError::NoSuchVariable { variable_identifier: "content".into() }.boxed())?;

        println!("{}", arg.to_string());

        Ok(Value::Null)
    }
}