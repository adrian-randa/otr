use crate::core::expression::Expression;

#[derive(Debug, Clone)]
pub enum BooleanExpression {
    And { lhs: Box<Expression>, rhs: Box<Expression> },
    Or { lhs: Box<Expression>, rhs: Box<Expression> },
    Not (Box<Expression>),
}