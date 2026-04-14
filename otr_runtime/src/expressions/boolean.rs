use otr_core::{expression::{Expression, boolean::BooleanExpression}, value::Value::{self, *}, error::Result};

use crate::{expressions::eval_expression, error::RuntimeError, environment::Environment};

pub(crate) fn eval_boolean_expression(expression: &BooleanExpression, environment: &Environment) -> Result<Value> {
    match expression {
        BooleanExpression::And { lhs, rhs } => eval_and_expression(lhs, rhs, environment),
        BooleanExpression::Or { lhs, rhs } => eval_or_expression(lhs, rhs, environment),
        BooleanExpression::Not(expression) => eval_not_expression(expression, environment),
    }
}

fn eval_and_expression(lhs: &Expression, rhs: &Expression, environment: &crate::Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Bool(l), Bool(r)) => Ok(Bool(l && r)),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot perform boolean and operation on {} and {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}

fn eval_or_expression(lhs: &Expression, rhs: &Expression, environment: &crate::Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Bool(l), Bool(r)) => Ok(Bool(l || r)),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot perform boolean or operation on {} and {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}

fn eval_not_expression(expression: &Expression, environment: &crate::Environment) -> Result<Value> {
    let value = eval_expression(expression, environment)?;

    match value {
        Bool(value) => Ok(Bool(!value)),

        value => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot perform boolean nor operation on {}!",
                value.get_type_id()
            ),
        }
        .boxed()),
    }
}
