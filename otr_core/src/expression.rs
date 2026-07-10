use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{{expression::{arithmetic::ArithmeticExpression, boolean::BooleanExpression, comparison::ComparisonExpression, variable::VariableExpression}, module::ModuleAddress, value::Value}};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Operator {
    Add, Subtract, Multiply, Divide, Modulo, Power, And, Or, Not, Greater
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", 
            match self {
                Operator::Add => "add",
                Operator::Subtract => "subtract",
                Operator::Multiply => "multiply",
                Operator::Divide => "divide",
                Operator::Modulo => "modulo",
                Operator::Power => "power",
                Operator::And => "and",
                Operator::Or => "or",
                Operator::Not => "not",
                Operator::Greater => "greater than",
            }
        )
    }
}

impl Deref for Operator {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl AsRef<Self> for Operator {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcedureCallExpression {
    procedure_id: ModuleAddress,
    arguments: Vec<Expression>,
}

impl ProcedureCallExpression {
    pub fn new(procedure_id: ModuleAddress, arguments: Vec<Expression>) -> Self {
        Self {
            procedure_id,
            arguments,
        }
    }

    pub fn get_procedure_id(&self) -> &ModuleAddress {
        &self.procedure_id
    }

    pub fn get_arguments(&self) -> &Vec<Expression> {
        &self.arguments
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssociatedProcedureCallExpression {
    callee_expression: Box<Expression>,
    procedure_ident: String,
    arguments: Vec<Expression>,
}

impl AssociatedProcedureCallExpression {
    pub fn new(callee_expression: Box<Expression>, procedure_ident: String, arguments: Vec<Expression>) -> Self {
        Self { callee_expression, procedure_ident, arguments }
    }

    pub fn get_callee_expression(&self) -> &Expression {
        &self.callee_expression
    }

    pub fn get_procedure_ident(&self) -> &String {
        &self.procedure_ident
    }

    pub fn get_arguments(&self) -> &Vec<Expression> {
        &self.arguments
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructMemberExpression {
    subexpression: Box<Expression>,
    member_ident: String,
}

impl StructMemberExpression {
    pub fn new(subexpression: Box<Expression>, member_ident: String) -> Self {
        Self { subexpression, member_ident }
    }

    pub fn get_subexpression(&self) -> &Expression {
        &self.subexpression
    }

    pub fn get_member_ident(&self) -> &String {
        &self.member_ident
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayIndexExpression {
    subexpression: Box<Expression>,
    index_expression: Box<Expression>,
}

impl ArrayIndexExpression {
    pub fn new(subexpression: Box<Expression>, index_expression: Box<Expression>) -> Self {
        Self { subexpression, index_expression }
    }

    pub fn get_subexpression(&self) -> &Expression {
        &self.subexpression
    }

    pub fn get_index_expression(&self) -> &Expression {
        &self.index_expression
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructConstructionExpression {
    struct_id: ModuleAddress,
    field_overrides: Vec<(String, Expression)>,
}

impl StructConstructionExpression {
    pub fn new(struct_id: ModuleAddress, field_overrides: Vec<(String, Expression)>) -> Self {
        Self { struct_id, field_overrides }
    }

    pub fn get_struct_id(&self) -> &ModuleAddress {
        &self.struct_id
    }

    pub fn get_field_overrides(&self) -> &Vec<(String, Expression)> {
        &self.field_overrides
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayConstructionExpression {
    items: Vec<Expression>,
}

impl ArrayConstructionExpression {
    pub fn new(items: Vec<Expression>) -> Self {
        Self { items }
    }

    pub fn get_items(&self) -> &Vec<Expression> {
        &self.items
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchExpression {
    subexpression: Box<Expression>,
}

impl CatchExpression {
    pub fn new(subexpression: Box<Expression>) -> Self {
        Self { subexpression }
    }

    pub fn get_subexpression(&self) -> &Expression {
        &self.subexpression
    }
}

pub mod variable;
pub mod arithmetic;
pub mod boolean;
pub mod comparison;