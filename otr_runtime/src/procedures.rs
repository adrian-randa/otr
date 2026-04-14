use std::collections::HashMap;


use otr_core::{procedure::{CompiledProcedure, Instruction}, value::Value, r#type::Type, error::Result};
use crate::{error::RuntimeError, environment::Environment, error::ValueError, expressions::eval_expression};

pub(crate) trait Procedure: std::fmt::Debug {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<Value>;
}


#[allow(unused)]
#[derive(Debug)]
pub(crate) enum RuntimeProcedure<'a> {
    Abstract(Box<dyn Procedure>),
    AbstractRef(&'a dyn Procedure),
    Compiled(Box<CompiledProcedure>),
    CompiledRef(&'a CompiledProcedure),
}

impl<'a> Procedure for RuntimeProcedure<'a> {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<Value> {
        match self {
            RuntimeProcedure::Abstract(procedure) => procedure.call(environment, arguments),
            RuntimeProcedure::Compiled(compiled_procedure) => call_compiled_procedure(compiled_procedure, environment, arguments),
            RuntimeProcedure::AbstractRef(procedure) => procedure.call(environment, arguments),
            RuntimeProcedure::CompiledRef(compiled_procedure) => call_compiled_procedure(*compiled_procedure, environment, arguments),
        }
    }
}


fn call_compiled_procedure(procedure: &CompiledProcedure, mut environment: Environment, arguments: Vec<Value>) -> Result<Value> {
    let members = HashMap::from_iter(
        procedure.get_argument_identifiers()
            .clone()
            .into_iter()
            .zip(arguments.into_iter()),
    );

    environment.insert_members(members);

    let mut pc = 0;
    let instructions = procedure.get_instructions();

    while pc < instructions.len() {
        match &instructions[pc] {
            Instruction::PushVarToScope { identifier } => {
                environment.get_scope_mut().push(identifier.clone())?;
            }
            Instruction::PopVarFromScope { identifier } => {
                environment.get_scope_mut().pop(identifier)?;
            }
            Instruction::GrowStack => {
                environment.get_scope_mut().grow_stack();
            }
            Instruction::ShrinkStack => {
                environment.get_scope_mut().shrink_stack();
            }
            Instruction::EvaluateExpression { expression, target } => {
                let eval_result = eval_expression(expression, &environment)?;

                if let Some(target) = target {
                    environment.set_variable(target.clone(), eval_result)?;
                }
            }
            Instruction::JumpConditional {
                condition_expression,
                jump_target,
            } => {
                let returned_value = eval_expression(condition_expression, &environment)?;

                match returned_value {
                    Value::Bool(value) => {
                        if value {
                            pc = *jump_target;
                            continue;
                        }
                    }
                    _ => {
                        return Err(RuntimeError::TypeMismatch {
                            expected: Type::Bool,
                            found: returned_value.get_type_id(),
                        }
                        .boxed())
                    }
                }
            }
            Instruction::Return { expression} => return eval_expression(expression, &environment),
            Instruction::Throw { expression } => {
                return Err(ValueError::new(eval_expression(expression, &environment)?).boxed())
            },
        }

        pc += 1;
    }

    Ok(Value::Null)
}