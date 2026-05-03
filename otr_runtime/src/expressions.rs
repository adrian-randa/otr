use std::{cell::RefCell, rc::Rc};

use otr_core::{error::Result, expression::{variable::{VariableAccessMode, VariableAddressant, VariableExpression}, *}, r#type::Type, value::Value};
use crate::{environment::Environment, error::{RuntimeError, context::{AssociatedProcedureContextDecorator, ProcedureContextDecorator}}, expressions::{arithmetic::eval_arithmetic_expression, boolean::eval_boolean_expression, comparison::eval_comparison_expression}, module::Module, procedures::Procedure, scope::Scope, value};


pub(crate) fn eval_expression(expression: &Expression, environment: &Environment) -> Result<Value> {
    match expression {
        Expression::Value(value) => Ok(value.clone()),
        Expression::Variable(expr) => eval_variable_expression(expr, environment),
        Expression::Arithmetic(expr) => eval_arithmetic_expression(expr, environment),
        Expression::Boolean(expr) => eval_boolean_expression(expr, environment),
        Expression::Comparison(expr) => eval_comparison_expression(expr, environment),
        Expression::ProcedureCall(expr) => eval_procedure_call(expr, environment),
        Expression::AssociatedProcedureCall(expr) => eval_associated_procedure_call_expression(expr, environment),
        Expression::StructConstruction(expr) => eval_struct_construction_expression(expr, environment),
        Expression::StructMember(expr) => eval_struct_member_expression(expr, environment),
        Expression::ArrayConstruction(expr) => eval_array_construction_expression(expr, environment),
        Expression::ArrayIndex(expr) => eval_array_index_expression(expr, environment),
        Expression::Catch(expr) => eval_catch_expression(expr, environment),
    }
}


fn eval_procedure_call(expression: &ProcedureCallExpression, environment: &Environment) -> Result<Value> {
    let procedure = environment.get_procedure_by_address(expression.get_procedure_id())?;

    let mut arguments = Vec::with_capacity(expression.get_arguments().len());
    for eval_result in expression
        .get_arguments()
        .iter()
        .map(|arg_exp| eval_expression(arg_exp, environment))
    {
        arguments.push(eval_result?);
    }

    let environment = environment.open_subenvironment(Scope::new(), &expression.get_procedure_id());

    Ok(procedure
        .call(environment, arguments)
        .map_err(|error| ProcedureContextDecorator {
            error,
            procedure_id: expression.get_procedure_id().clone()
        }.boxed())
        ?
    )
}


fn eval_struct_construction_expression(expression: &StructConstructionExpression, environment: &Environment) -> Result<Value> {
    let mut instance = environment.get_struct_by_address(&expression.get_struct_id())?;

    for (field, expr) in expression.get_field_overrides() {
        let value = eval_expression(expr, environment)?;
        instance.get_members_mut().set(field, value)
            .ok_or(RuntimeError::NoSuchMember { member_identifier: field.clone() }.boxed())?;
    }

    Ok(Value::Struct(Rc::new(RefCell::new(Some(instance)))))
}


fn eval_array_construction_expression(expression: &ArrayConstructionExpression, environment: &Environment) -> Result<Value> {
    let mut array = Vec::with_capacity(expression.get_items().len());
    for item in expression.get_items().iter() {
        array.push(eval_expression(item, environment)?);
    }
    Ok(Value::Array(array))
}


fn eval_variable_expression(expression: &VariableExpression, environment: &Environment) -> Result<Value> {
    use VariableAccessMode::*;

    match expression.get_access_mode() {
        Move => environment.query_variable(expression.get_address().clone()),
        Clone => environment.clone_variable(expression.get_address().clone()),
        Ref => environment.reference_variable(expression.get_address().clone()),
        TypeOf => environment.get_variable_type(expression.get_address().clone()),
    }
}


fn eval_struct_member_expression(expression: &StructMemberExpression, environment: &Environment) -> Result<Value> {
    let st = eval_expression(expression.get_subexpression(), environment)?;
    value::get(
        &st,
        vec![VariableAddressant::Identifier(expression.get_member_ident().clone())].into_iter(),
        environment.get_contained_module_id(),
    )
}

fn eval_array_index_expression(expression: &ArrayIndexExpression, environment: &Environment) -> Result<Value> {
    let index = eval_expression(expression.get_index_expression(), environment)?;

    let index = if let Value::Integer(index) = index {
        index
    } else {
        return Err(RuntimeError::TypeMismatch {
            expected: Type::Integer,
            found: index.get_type_id(),
        }
        .boxed());
    };

    let array = eval_expression(expression.get_subexpression(), environment)?;
    value::get(
        &array,
        vec![VariableAddressant::Index(index.try_into().unwrap())].into_iter(),
        environment.get_contained_module_id(),
    )
}


fn eval_associated_procedure_call_expression(expression: &AssociatedProcedureCallExpression, environment: &Environment) -> Result<Value> {
    let callee = eval_expression(expression.get_callee_expression(), environment)?;

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
                procedure_identifier: expression.get_procedure_ident().clone(),
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
        &expression.get_procedure_ident(),
        environment.get_contained_module_id() == callee_id.get_module_id(),
    )?;

    let mut arguments = Vec::with_capacity(expression.get_arguments().len() + 1);
    arguments.push(callee);
    for eval_result in expression
        .get_arguments()
        .iter()
        .map(|arg_exp| eval_expression(arg_exp, environment))
    {
        arguments.push(eval_result?);
    }

    let environment = environment.open_subenvironment(Scope::new(), &callee_id);

    Ok(procedure.call(environment, arguments).map_err(|error| {
        AssociatedProcedureContextDecorator::new_boxed(
            error,
            callee_id,
            expression.get_procedure_ident().clone(),
        )
    })?)
}


fn eval_catch_expression(expression: &CatchExpression, environment: &Environment) -> Result<Value> {
    Ok(
        match eval_expression(expression.get_subexpression(), environment) {
            Ok(value) => value,
            Err(err) => err.to_value(),
        }
    )
}

pub mod arithmetic;
pub mod boolean;
pub mod comparison;