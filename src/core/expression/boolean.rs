use serde::{Deserialize, Serialize};

use crate::core::expression::Expression;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BooleanExpression {
    And { lhs: Box<Expression>, rhs: Box<Expression> },
    Or { lhs: Box<Expression>, rhs: Box<Expression> },
    Not (Box<Expression>),
}