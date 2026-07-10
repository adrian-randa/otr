use otr_core::{error::Result, expression::{Expression, Operator, arithmetic::ArithmeticExpression}, value::Value::{self, *}};

use crate::{environment::Environment, error::RuntimeError, expressions::{eval_overloaded_operator_expression_owned, eval_overloaded_operator_expression_ref}, module::Module, procedures::Procedure, scope::Scope};

use super::eval_expression;

pub(crate) fn eval_arithmetic_expression(expression: &ArithmeticExpression, environment: &Environment) -> Result<Value> {
    match expression {
        ArithmeticExpression::Add { lhs, rhs } => eval_add_expression(lhs, rhs, environment),
        ArithmeticExpression::Subtract { lhs, rhs } => eval_subtract_expression(lhs, rhs, environment),
        ArithmeticExpression::Multiply { lhs, rhs } => eval_multiply_expression(rhs, lhs, environment),
        ArithmeticExpression::Divide { lhs, rhs } => eval_divide_expression(lhs, rhs, environment),
        ArithmeticExpression::Power { base, exponent } => eval_power_expression(base, exponent, environment),
        ArithmeticExpression::Modulo { lhs, rhs } => eval_modulo_expression(lhs, rhs, environment),
    }
}

fn eval_add_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Integer(l), Integer(r)) => Ok(Integer(l + r)),
        (Float(l), Float(r)) => Ok(Float(l + r)),

        (String(l), String(r)) => Ok(String(l.to_string() + &r)),

        (String(l), Integer(r)) => Ok(String(l + &r.to_string())),
        (String(l), Float(r)) => Ok(String(l + &r.to_string())),
        (Integer(l), String(r)) => Ok(String(l.to_string() + &r)),
        (Float(l), String(r)) => Ok(String(l.to_string() + &r)),

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::Add, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::Add, r, environment),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!("Cannot add {} and {}!", l.get_type_id(), r.get_type_id()),
        }
        .boxed()),
    }
}


fn eval_subtract_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Integer(l), Integer(r)) => Ok(Integer(l - r)),
        (Float(l), Float(r)) => Ok(Float(l - r)),

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::Subtract, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::Subtract, r, environment),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot subtract {} and {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}

fn eval_multiply_expression(rhs: &Expression, lhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Integer(l), Integer(r)) => Ok(Integer(l * r)),
        (Float(l), Float(r)) => Ok(Float(l * r)),

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::Multiply, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::Multiply, r, environment),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot multiply {} and {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}


fn eval_divide_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Integer(l), Integer(r)) => {
            if r == 0 {
                Err(Box::new(RuntimeError::DivisionByZero))
            } else {
                Ok(Value::Integer(l / r))
            }
        },
        (Float(l), Float(r)) => Ok(Float(l / r)),

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::Divide, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::Divide, r, environment),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!("Cannot divide {} and {}!", l.get_type_id(), r.get_type_id()),
        }
        .boxed()),
    }
}

fn eval_power_expression(base: &Expression, exponent: &Expression, environment: &Environment) -> Result<Value> {
    let base = eval_expression(base, environment)?;
    let exponent = eval_expression(exponent, environment)?;

    match (base, exponent) {
        (Integer(l), Integer(r)) => Ok(Integer(
            l.checked_pow(r.try_into().map_err(|_| {
                RuntimeError::Unknown {
                    message: "Could not compute power; the exponent was too large!".into(),
                }
                .boxed()
            })?)
            .ok_or(
                RuntimeError::Unknown {
                    message: "Overflow occured while computing power!".into(),
                }
                .boxed(),
            )?,
        )),
        (Float(l), Float(r)) => Ok(Float(l.powf(r))),

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::Power, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::Power, r, environment),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot compute power of {} and {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}

fn eval_modulo_expression(lhs: &Expression, rhs: &Expression, environment: &Environment) -> Result<Value> {
    let lhs = eval_expression(lhs, environment)?;
    let rhs = eval_expression(rhs, environment)?;

    match (lhs, rhs) {
        (Integer(l), Integer(r)) => Ok(Integer(l.rem_euclid(r))),
        (Float(l), Float(r)) => Ok(Float(l.rem_euclid(r))),

        (Struct(l), r) => eval_overloaded_operator_expression_owned(l, Operator::Modulo, r, environment),
        (StructRef(l), r) => eval_overloaded_operator_expression_ref(l, Operator::Modulo, r, environment),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Cannot modulate {} by {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}