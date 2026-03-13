

pub(crate) enum Expression {
    Value(Value),
    Arithmetic(ArithmeticExpression),
    Boolean(BooleanExpression),
    Comparison(ComparisonExpression),
    ProcedureCall(ProcedureCallExpression),
    StructConstruction(StructConstructionExpression),
}

#[derive(Debug)]
pub struct ProcedureCallExpression {
    procedure_id: ModuleAddress,
    arguments: Vec<Expression>,
}

impl ErrorContextualizer for ProcedureCallExpression {
    fn contextualize(&self, error: Box<dyn Error>) -> Box<dyn Error> {
        ProcedureContextDecorator::new_boxed(error, self.procedure_id.clone())
    }
}

pub mod arithmetic;
pub mod boolean;
pub mod comparison;