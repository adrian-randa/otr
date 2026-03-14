use crate::core::expression::comparison::ComparisonExpression;
use crate::error::Result;
use crate::core::{value::Value::{self, *}, expression::Expression};
use crate::error::runtime_error::RuntimeError;
use crate::runtime::environment::Environment;
use crate::runtime::expressions::eval_expression;

pub(crate) fn eval_comparison_expression(expression: &ComparisonExpression, environment: &Environment) -> Result<Value> {
    match expression {
        ComparisonExpression::Equals { lhs, rhs } => eval_equals_expression(lhs, rhs, environment),
        ComparisonExpression::GreaterThan { lhs, rhs } => eval_greater_than_expression(lhs, rhs, environment),
    }
}

fn eval_greater_than_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Integer(l), Integer(r)) => Ok(Bool(l > r)),
        (Float(l), Float(r)) => Ok(Bool(l > r)),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Ordering is undefined on {} and {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}

fn eval_equals_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    Ok(Bool(lhs == rhs))
}