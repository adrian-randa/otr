use std::{cell::RefCell, rc::Rc};

use crate::{
    error::{
        context::{AssociatedProcedureContextDecorator, ProcedureContextDecorator},
        Error, ErrorContextualizer,
    },
    runtime::{
        module::Module,
        scope::{Scope, ScopeAddress, ScopeAddressant},
        Environment, Expression, ModuleAddress, RuntimeError, Value,
    },
};

use crate::error::Result;

#[derive(Debug)]
pub struct ProcedureCallExpression {
    //TODO: Remove public visibility
    pub procedure_id: ModuleAddress,
    pub arguments: Vec<Box<dyn Expression>>,
}

impl ErrorContextualizer for ProcedureCallExpression {
    fn contextualize(&self, error: Box<dyn Error>) -> Box<dyn Error> {
        ProcedureContextDecorator::new_boxed(error, self.procedure_id.clone())
    }
}

impl Expression for ProcedureCallExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        let procedure = environment.get_procedure_by_address(&self.procedure_id)?;

        let mut arguments = Vec::with_capacity(self.arguments.len());
        for eval_result in self
            .arguments
            .iter()
            .map(|arg_exp| arg_exp.eval(environment))
        {
            arguments.push(eval_result?);
        }

        let environment = environment.open_subenvironment(Scope::new(), &self.procedure_id);

        Ok(procedure
            .call(environment, arguments)
            .map_err(|error| self.contextualize(error))?)
    }
}

impl ProcedureCallExpression {
    pub(crate) fn new(procedure_id: ModuleAddress, arguments: Vec<Box<dyn Expression>>) -> Self {
        Self {
            procedure_id,
            arguments,
        }
    }
}

#[derive(Debug)]
pub struct StructConstructionExpression {
    pub struct_id: ModuleAddress,
    pub field_overrides: Vec<(String, Box<dyn Expression>)>,
}

impl Expression for StructConstructionExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        let mut instance = environment.get_struct_by_address(&self.struct_id)?;

        for (field, expr) in &self.field_overrides {
            let value = expr.eval(environment)?;
            instance.get_members_mut().set_member(field, value)?;
        }

        Ok(Value::Struct(Rc::new(RefCell::new(Some(instance)))))
    }
}

#[derive(Debug)]
pub struct ArrayConstructionExpression {
    pub items: Vec<Box<dyn Expression>>,
}

impl Expression for ArrayConstructionExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        let mut array = Vec::with_capacity(self.items.len());
        for item in self.items.iter() {
            array.push(item.eval(environment)?);
        }
        Ok(Value::Array(array))
    }
}

#[derive(Debug)]
pub struct VariableExpression {
    //TODO: Change visibility to private
    pub variable_address: ScopeAddress,
}

impl Expression for VariableExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        environment.query_variable(self.variable_address.clone())
    }
}

#[derive(Debug)]
pub struct ReferenceExpression {
    pub variable_address: ScopeAddress,
}

impl Expression for ReferenceExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        environment.reference_variable(self.variable_address.clone())
    }
}

#[derive(Debug)]
pub struct CloneExpression {
    pub variable_address: ScopeAddress,
}

impl Expression for CloneExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        environment.clone_variable(self.variable_address.clone())
    }
}

#[derive(Debug)]
pub struct EqualityExpression {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>,
}

impl EqualityExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self { lhs, rhs }
    }
}

impl Expression for EqualityExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        use super::Value::*;

        let lhs = self.lhs.eval(environment)?;
        let rhs = self.rhs.eval(environment)?;

        Ok(Bool(lhs == rhs))
    }
}

#[derive(Debug)]
pub(crate) struct TypeofVariableExpression {
    pub(crate) variable_address: ScopeAddress,
}

impl Expression for TypeofVariableExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        environment.get_variable_type(self.variable_address.clone())
    }
}

#[derive(Debug)]
pub(crate) struct StructMemberExpression {
    pub(crate) subexpression: Box<dyn Expression>,
    pub(crate) member_ident: String,
}

impl Expression for StructMemberExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        self.subexpression.eval(environment)?.query(
            vec![ScopeAddressant::Identifier(self.member_ident.clone())],
            environment.get_contained_module_id(),
        )
    }
}

#[derive(Debug)]
pub(crate) struct ArrayIndexExpression {
    pub(crate) subexpression: Box<dyn Expression>,
    pub(crate) index_expression: Box<dyn Expression>,
}

impl Expression for ArrayIndexExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        let index = self.index_expression.eval(environment)?;

        let index = if let Value::Integer(index) = index {
            index
        } else {
            return Err(RuntimeError::TypeMismatch {
                expected: crate::runtime::Type::Integer,
                found: index.get_type_id(),
            }
            .boxed());
        };

        self.subexpression.eval(environment)?.query(
            vec![ScopeAddressant::Index(index.try_into().unwrap())],
            environment.get_contained_module_id(),
        )
    }
}

#[derive(Debug)]
pub(crate) struct AssociatedProcedureCallExpression {
    pub(crate) callee_expression: Box<dyn Expression>,
    pub(crate) procedure_ident: String,
    pub(crate) arguments: Vec<Box<dyn Expression>>,
}

impl Expression for AssociatedProcedureCallExpression {
    fn eval(&self, environment: &Environment) -> Result<Value> {
        let callee = self.callee_expression.eval(environment)?;

        let callee_id = match &callee {
            Value::Struct(s) => s
                .borrow()
                .as_ref()
                .ok_or(RuntimeError::UseOfMovedValue.boxed())?
                .get_struct_id()
                .clone(),
            Value::StructRef(s) => s
                .upgrade()
                .ok_or(RuntimeError::UseOfDroppedValue.boxed())?
                .borrow()
                .as_ref()
                .ok_or(RuntimeError::UseOfMovedValue.boxed())?
                .get_struct_id()
                .clone(),
            other => {
                return Err(RuntimeError::AssociatedProcedureNotDefined {
                    procedure_identifier: self.procedure_ident.clone(),
                    struct_identifier: other.get_type_id().to_string(),
                }
                .boxed());
            }
        };

        let module = environment
            .get_loaded_module(callee_id.get_module_id())
            .ok_or(
                RuntimeError::ModuleNotLoaded {
                    module_identifier: callee_id.get_module_id().clone(),
                }
                .boxed(),
            )?;

        let procedure = module.get_associated_procedure(
            callee_id.get_identifier(),
            &self.procedure_ident,
            environment.get_contained_module_id() == callee_id.get_module_id(),
        )?;

        let mut arguments = Vec::with_capacity(self.arguments.len() + 1);
        arguments.push(callee);
        for eval_result in self
            .arguments
            .iter()
            .map(|arg_exp| arg_exp.eval(environment))
        {
            arguments.push(eval_result?);
        }

        let environment = environment.open_subenvironment(Scope::new(), &callee_id);

        Ok(procedure.call(environment, arguments).map_err(|error| {
            AssociatedProcedureContextDecorator::new_boxed(
                error,
                callee_id,
                self.procedure_ident.clone(),
            )
        })?)
    }
}

pub mod arithmetic;
pub mod boolean;
