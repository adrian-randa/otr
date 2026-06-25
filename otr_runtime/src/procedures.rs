use otr_core::{error::Result, procedure::{CompiledProcedure, Instruction}, r#type::Type, value::Value};
use crate::{error::RuntimeError, environment::Environment, error::ValueError, expressions::eval_expression};

pub(crate) trait Procedure: std::fmt::Debug {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<Value>;

    fn get_num_args(&self) -> usize;
    fn get_stack_size(&self) -> usize;
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
            RuntimeProcedure::CompiledRef(compiled_procedure) => call_compiled_procedure(compiled_procedure, environment, arguments),
        }
    }
    
    fn get_num_args(&self) -> usize {
        match self {
            RuntimeProcedure::Abstract(procedure) => procedure.get_num_args(),
            RuntimeProcedure::AbstractRef(procedure) => procedure.get_num_args(),
            RuntimeProcedure::Compiled(compiled_procedure) => compiled_procedure.num_args,
            RuntimeProcedure::CompiledRef(compiled_procedure) => compiled_procedure.num_args,
        }
    }
    
    fn get_stack_size(&self) -> usize {
        match self {
            RuntimeProcedure::Abstract(procedure) => procedure.get_stack_size(),
            RuntimeProcedure::AbstractRef(procedure) => procedure.get_stack_size(),
            RuntimeProcedure::Compiled(compiled_procedure) => compiled_procedure.stack_size,
            RuntimeProcedure::CompiledRef(compiled_procedure) => compiled_procedure.stack_size,
        }
    }
}


fn call_compiled_procedure(procedure: &CompiledProcedure, mut environment: Environment, arguments: Vec<Value>) -> Result<Value> {
    for (i, argument) in arguments.into_iter().enumerate() {
        environment.get_scope_mut().set(i, argument);
    }

    let mut pc = 0;
    let instructions = &procedure.instructions;

    while pc < instructions.len() {
        match &instructions[pc] {
            Instruction::PushVarToScope { identifier: _ } => {
                // Legacy; Noop
            }
            Instruction::PopVarFromScope { identifier: _ } => {
                // Legacy; Noop
            }
            Instruction::GrowStack => {
                // Legacy; Noop
            }
            Instruction::ShrinkStack => {
                // Legacy; Noop
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