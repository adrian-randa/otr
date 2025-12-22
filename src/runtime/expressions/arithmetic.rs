use crate::runtime::{expressions::Expression, Environment, RuntimeError};

#[derive(Debug)]
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

            (String(l), Integer(r)) => Ok(String(l + &r.to_string())),
            (String(l), Float(r)) => Ok(String(l + &r.to_string())),
            (Integer(l), String(r)) => Ok(String(l.to_string() + &r)),
            (Float(l), String(r)) => Ok(String(l.to_string() + &r)),

            (l, r) => Err(RuntimeError {
                message: format!("Cannot add {} and {}!", l.get_type_id(), r.get_type_id()),
            }),
        }
    }
}

#[derive(Debug)]
pub struct SubtractExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl SubtractExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for SubtractExpression {
    fn eval(&self, environment: &Environment) -> Result<crate::runtime::Value, RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {
            (Integer(l), Integer(r)) => Ok(Integer(l - r)),
            (Float(l), Float(r)) => Ok(Float(l - r)),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Cannot subtract {} and {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct MultiplyExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl MultiplyExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for MultiplyExpression {
    fn eval(&self, environment: &Environment) -> Result<crate::runtime::Value, RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {
            (Integer(l), Integer(r)) => Ok(Integer(l * r)),
            (Float(l), Float(r)) => Ok(Float(l * r)),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Cannot multiply {} and {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct DivideExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl DivideExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for DivideExpression {
    fn eval(&self, environment: &Environment) -> Result<crate::runtime::Value, RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {
            (Integer(l), Integer(r)) => Ok(Integer(l / r)),
            (Float(l), Float(r)) => Ok(Float(l / r)),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Cannot divide {} and {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct PowerExpression {
    base: Box<dyn Expression>,
    exponent: Box<dyn Expression>,
}

impl PowerExpression {
    pub fn new(base: Box<dyn Expression>, exponent: Box<dyn Expression>) -> Self {
        Self { base, exponent }
    }
}

impl Expression for PowerExpression {
    fn eval(&self, environment: &Environment) -> Result<crate::runtime::Value, RuntimeError> {
        use super::Value::*;

        let base = self.base.eval(environment)?;
        let exponent = self.exponent.eval(environment)?;

        match (base, exponent) {
            (Integer(l), Integer(r)) => Ok(Integer(
                l.checked_pow(r.try_into().map_err(|_| RuntimeError {
                    message: "Could not compute power; the exponent was too large!".into(),
                })?)
                .ok_or(RuntimeError {
                    message: "Overflow occured while computing power!".into(),
                })?,
            )),
            (Float(l), Float(r)) => Ok(Float(l.powf(r))),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Cannot compute power of {} and {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct ModuloExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl ModuloExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for ModuloExpression {
    fn eval(&self, environment: &Environment) -> Result<crate::runtime::Value, RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {
            (Integer(l), Integer(r)) => Ok(Integer(l.rem_euclid(r))),
            (Float(l), Float(r)) => Ok(Float(l.rem_euclid(r))),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Cannot modulate {} by {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct GreaterThanExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl GreaterThanExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for GreaterThanExpression {
    fn eval(&self, environment: &Environment) -> Result<crate::runtime::Value, RuntimeError> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        match (lhs, rhs) {
            (Integer(l), Integer(r)) => Ok(Bool(l > r)),
            (Float(l), Float(r)) => Ok(Bool(l > r)),

            (l, r) => Err(RuntimeError {
                message: format!(
                    "Ordering is undefined on {} and {}!",
                    l.get_type_id(),
                    r.get_type_id()
                ),
            }),
        }
    }
}