use std::{collections::HashMap, rc::Rc};

use crate::runtime::{Environment, RuntimeError, Value};


pub trait Expression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError>;
}


pub struct AddExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl AddExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for AddExpression {
    fn eval(&self, environment: &Environment) -> Result<super::Value, RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {

            (Integer(l), Integer(r)) => Ok(Integer(l + r)),
            (Float(l), Float(r)) => Ok(Float(l + r)),
            (String(l), String(r)) => Ok(String(l.to_string() + &r)),

            (l, r) => Err(RuntimeError { message: format!("Cannot add {} and {}!", l.get_type_id(), r.get_type_id()) })
        }
    }
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


pub struct VariableExpression {
    scope: Rc<Vec<Value>>,
    variable_index: usize,
}

impl Expression for VariableExpression {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError> {
        self.scope
            .get(self.variable_index)
            .map(|value| value.clone())
            .ok_or(RuntimeError {
                message: "The variable, whose value was supposed to be sampled, is out of scope!".into()
            } )
    }
}