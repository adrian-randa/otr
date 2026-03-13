
pub(crate) enum BooleanExpression {
    And { lhs: Box<Expression>, rhs: Box<Expression> },
    Or { lhs: Box<Expression>, rhs: Box<Expression> },
    Not (Box<Expression>),
}