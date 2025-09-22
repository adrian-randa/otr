use crate::runtime::{Environment, RuntimeError, ScopeAddress, Value};


pub trait Expression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError>;
}



pub struct ProcedureCallExpression { //TODO: Remove public visibility
    pub procedure_id: String,
    pub arguments: Vec<Box<dyn Expression>>
}

impl Expression for ProcedureCallExpression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError> {
        let procedure = environment.get_procedure_by_id(&self.procedure_id)?;
        
        let mut arguments = Vec::with_capacity(self.arguments.len());
        for eval_result in self.arguments
            .iter()
            .map(|arg_exp| {
                arg_exp.eval(environment)
            }) {
                arguments.push(eval_result?);
            }

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
    fn eval(&self, environment: &crate::runtime::Environment) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        Ok(Bool(lhs == rhs))
    }
}

pub mod arithmetic;
pub mod boolean;