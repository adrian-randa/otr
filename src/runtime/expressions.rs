use std::cell::RefCell;

use crate::runtime::{Environment, ModuleAddress, RuntimeError, Scope, ScopeAddress, Value};


pub trait Expression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError>;
}



pub struct ProcedureCallExpression { //TODO: Remove public visibility
    pub procedure_id: ModuleAddress,
    pub arguments: Vec<Box<dyn Expression>>
}

impl Expression for ProcedureCallExpression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError> {
        let procedure = environment.get_procedure_by_address(&self.procedure_id)?;
        
        let mut arguments = Vec::with_capacity(self.arguments.len());
        for eval_result in self.arguments
            .iter()
            .map(|arg_exp| {
                arg_exp.eval(environment)
            }) {
                arguments.push(eval_result?);
            }
        
        let mut environment = environment.clone_with_scope(Scope::new());
        environment.set_contained_module(self.procedure_id.get_module_id().into());

        Ok(procedure.call(environment, arguments)?)
    }
}



pub struct VariableExpression { //TODO: Change visibility to private
    pub variable_address: ScopeAddress,
}

impl Expression for VariableExpression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError> {
        environment.lookup_variable(self.variable_address.clone())
    }
}



pub struct EqualityExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl EqualityExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for EqualityExpression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        Ok(Bool(lhs == rhs))
    }
}

pub mod arithmetic;
pub mod boolean;