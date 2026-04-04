use serde::{Deserialize, Serialize};

use crate::{core::{expression::{arithmetic::ArithmeticExpression, boolean::BooleanExpression, comparison::ComparisonExpression, variable::VariableExpression}, module::ModuleAddress, value::Value}, error::{Error, ErrorContextualizer, context::ProcedureContextDecorator}};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Value(Value),
    Variable(VariableExpression),
    Arithmetic(ArithmeticExpression),
    Boolean(BooleanExpression),
    Comparison(ComparisonExpression),
    ProcedureCall(ProcedureCallExpression),
    AssociatedProcedureCall(AssociatedProcedureCallExpression),
    StructConstruction(StructConstructionExpression),
    StructMember(StructMemberExpression),
    ArrayConstruction(ArrayConstructionExpression),
    ArrayIndex(ArrayIndexExpression),
    Catch(CatchExpression)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureCallExpression {
    procedure_id: ModuleAddress,
    arguments: Vec<Expression>,
}

impl ErrorContextualizer for ProcedureCallExpression {
    fn contextualize(&self, error: Box<dyn Error>) -> Box<dyn Error> {
        ProcedureContextDecorator::new_boxed(error, self.procedure_id.clone())
    }
}

impl ProcedureCallExpression {
    pub(crate) fn new(procedure_id: ModuleAddress, arguments: Vec<Expression>) -> Self {
        Self {
            procedure_id,
            arguments,
        }
    }

    pub(crate) fn get_procedure_id(&self) -> &ModuleAddress {
        &self.procedure_id
    }

    pub(crate) fn get_arguments(&self) -> &Vec<Expression> {
        &self.arguments
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedProcedureCallExpression {
    callee_expression: Box<Expression>,
    procedure_ident: String,
    arguments: Vec<Expression>,
}

impl AssociatedProcedureCallExpression {
    pub(crate) fn new(callee_expression: Box<Expression>, procedure_ident: String, arguments: Vec<Expression>) -> Self {
        Self { callee_expression, procedure_ident, arguments }
    }

    pub(crate) fn get_callee_expression(&self) -> &Expression {
        &self.callee_expression
    }

    pub(crate) fn get_procedure_ident(&self) -> &String {
        &self.procedure_ident
    }

    pub(crate) fn get_arguments(&self) -> &Vec<Expression> {
        &self.arguments
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructMemberExpression {
    subexpression: Box<Expression>,
    member_ident: String,
}

impl StructMemberExpression {
    pub(crate) fn new(subexpression: Box<Expression>, member_ident: String) -> Self {
        Self { subexpression, member_ident }
    }

    pub(crate) fn get_subexpression(&self) -> &Box<Expression> {
        &self.subexpression
    }

    pub(crate) fn get_member_ident(&self) -> &String {
        &self.member_ident
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayIndexExpression {
    subexpression: Box<Expression>,
    index_expression: Box<Expression>,
}

impl ArrayIndexExpression {
    pub(crate) fn new(subexpression: Box<Expression>, index_expression: Box<Expression>) -> Self {
        Self { subexpression, index_expression }
    }

    pub(crate) fn get_subexpression(&self) -> &Box<Expression> {
        &self.subexpression
    }

    pub(crate) fn get_index_expression(&self) -> &Box<Expression> {
        &self.index_expression
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructConstructionExpression {
    struct_id: ModuleAddress,
    field_overrides: Vec<(String, Expression)>,
}

impl StructConstructionExpression {
    pub(crate) fn new(struct_id: ModuleAddress, field_overrides: Vec<(String, Expression)>) -> Self {
        Self { struct_id, field_overrides }
    }

    pub(crate) fn get_struct_id(&self) -> &ModuleAddress {
        &self.struct_id
    }

    pub(crate) fn get_field_overrides(&self) -> &Vec<(String, Expression)> {
        &self.field_overrides
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayConstructionExpression {
    items: Vec<Expression>,
}

impl ArrayConstructionExpression {
    pub(crate) fn new(items: Vec<Expression>) -> Self {
        Self { items }
    }

    pub(crate) fn get_items(&self) -> &Vec<Expression> {
        &self.items
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchExpression {
    subexpression: Box<Expression>,
}

impl CatchExpression {
    pub(crate) fn new(subexpression: Box<Expression>) -> Self {
        Self { subexpression }
    }

    pub(crate) fn get_subexpression(&self) -> &Box<Expression> {
        &self.subexpression
    }
}

pub mod variable;
pub mod arithmetic;
pub mod boolean;
pub mod comparison;