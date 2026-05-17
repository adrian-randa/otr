use otr_core::{expression::comparison::ComparisonExpression, error::Result, value::Value::{self, *}, expression::Expression};
use crate::environment::Environment;
use crate::expressions::eval_expression;

pub(crate) fn eval_comparison_expression(expression: &ComparisonExpression, environment: &Environment) -> Result<Value> {
    match expression {
        ComparisonExpression::Equals { lhs, rhs } => eval_equals_expression(lhs, rhs, environment),
        ComparisonExpression::GreaterThan { lhs, rhs } => eval_greater_than_expression(lhs, rhs, environment),
    }
}

fn eval_greater_than_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    Ok(Bool(crate::value::compare(&lhs, &rhs)?.is_gt()))
}

fn eval_equals_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    Ok(Bool(lhs == rhs))
}