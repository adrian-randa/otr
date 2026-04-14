use serde::{Deserialize, Serialize};

use crate::expression::Expression;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArithmeticExpression {
    Add { lhs: Box<Expression>, rhs: Box<Expression> },
    Subtract { lhs: Box<Expression>, rhs: Box<Expression> },
    Multiply { lhs: Box<Expression>, rhs: Box<Expression> },
    Divide { lhs: Box<Expression>, rhs: Box<Expression> },
    Power { base: Box<Expression>, exponent: Box<Expression> },
    Modulo { lhs: Box<Expression>, rhs: Box<Expression> },
}