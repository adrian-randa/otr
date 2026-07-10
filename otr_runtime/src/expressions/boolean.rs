use otr_core::{error::Result, expression::{Expression, Operator, boolean::BooleanExpression}, value::Value::{self, *}};

use crate::{environment::Environment, error::RuntimeError, expressions::{eval_expression, eval_overloaded_operator_expression_owned, eval_overloaded_operator_expression_ref}};

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

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::And, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::And, r, environment),

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

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::Or, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::Or, r, environment),

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

        Struct(l) => eval_overloaded_operator_expression_owned(l, Operator::Not, Value::Null, environment),
        StructRef(l) => eval_overloaded_operator_expression_ref(l, Operator::Not, Value::Null, environment),

        value => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot perform boolean nor operation on {}!",
                value.get_type_id()
            ),
        }
        .boxed()),
    }
}
