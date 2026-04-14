use serde::{Deserialize, Serialize};

use crate::expression::Expression;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonExpression {
    Equals { lhs: Box<Expression>, rhs: Box<Expression> },
    GreaterThan { lhs: Box<Expression>, rhs: Box<Expression> },
}