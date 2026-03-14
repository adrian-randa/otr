use crate::core::expression::Expression;


#[derive(Debug, Clone)]
pub enum ComparisonExpression {
    Equals { lhs: Box<Expression>, rhs: Box<Expression> },
    GreaterThan { lhs: Box<Expression>, rhs: Box<Expression> },
}