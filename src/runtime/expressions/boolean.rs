use crate::runtime::{expressions::Expression, RuntimeError};

#[derive(Debug)]
pub struct AndExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl AndExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for AndExpression {
    fn eval(
        &self,
        environment: &crate::runtime::Environment,
    ) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {
            (Bool(l), Bool(r)) => Ok(Bool(l && r)),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Cannot perform boolean and operation on {} and {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct OrExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl OrExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for OrExpression {
    fn eval(
        &self,
        environment: &crate::runtime::Environment,
    ) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {
            (Bool(l), Bool(r)) => Ok(Bool(l || r)),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Cannot perform boolean or operation on {} and {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct NotExpression {
    expr: Box<dyn Expression>,
}

impl NotExpression {
    pub fn new(expr: Box<dyn Expression>) -> Self {
        Self { expr }
    }
}

impl Expression for NotExpression {
    fn eval(
        &self,
        environment: &crate::runtime::Environment,
    ) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
        use super::Value::*;

        let value = self.expr.eval(environment)?;

        match value {
            Bool(value) => Ok(Bool(!value)),

            value => Err(RuntimeError {
                message: format!(
                    "Cannot perform boolean nor operation on {}!",
                    value.get_type_id()
                ),
            }),
        }
    }
}
