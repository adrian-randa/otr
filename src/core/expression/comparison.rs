

pub(crate) enum ComparisonExpression {
    Equals { lhs: Box<Expression>, rhs: Box<Expression> },
    GreaterThan { lhs: Box<Expression>, rhs: Box<Expression> },
}