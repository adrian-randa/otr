use std::{collections::HashMap, rc::Rc};

use crate::runtime::{expressions::Expression, Environment, RuntimeError, Scope, ScopeAddress, ScopeAddressant, Value};


pub trait Procedure {
    fn call(&self, environment: &Environment, arguments: Vec<Value>) -> Result<Value, RuntimeError>;
}



pub enum Instruction { //TODO: Remove public viisibility
    PushVarToScope {
        identifier: String,
    },
    PopVarFromScope {
        identifier: String,
    },
    EvaluateExpression {
        expression: Box<dyn Expression>,
        target: Option<ScopeAddress>,
    },
    JumpConditional{
        condition_expression: Box<dyn Expression>,
        jump_target: usize,
    },
    Return{
        expression: Box<dyn Expression>,
    },
}

pub struct CompiledProcedure { //TODO: Remove public visibility
    pub arguments_identifiers: Vec<String>,
    pub instructions: Vec<Instruction>
}

impl Procedure for CompiledProcedure {
    fn call(&self, environment: &Environment, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let members = HashMap::from_iter(self.arguments_identifiers
            .clone()
            .into_iter()
            .zip(arguments.into_iter())
        );

        let mut environment = environment.clone_with_scope(Scope::from_members(members));

        let mut pc = 0;

        while pc < self.instructions.len() {
            println!("{:?}", environment.scope);

            match &self.instructions[pc] {
                Instruction::PushVarToScope { identifier } => {
                    environment.scope.push(identifier.clone());
                },
                Instruction::PopVarFromScope { identifier } => {
                    environment.scope.pop(identifier);
                },
                Instruction::EvaluateExpression { expression, target } => {
                    let eval_result = expression.eval(&mut environment)?;
                    
                    if let Some(target) = target {
                        environment.set_variable(target.clone(), eval_result)?;
                    }
                },
                Instruction::JumpConditional { condition_expression: procedure, jump_target } => {
                    let returned_value = procedure.eval(&mut environment)?;

                    match returned_value {
                        Value::Bool(value) => {
                            if value {
                                pc = *jump_target;
                                continue;
                            }
                        },
                        _ => return Err(RuntimeError {
                            message: format!("Expected bool, found {}!", returned_value.get_type_id())
                        })
                    }
                },
                Instruction::Return { expression: procedure } => {
                    return procedure.eval(&mut environment)
                },
            }

            pc += 1;
        }

        Ok(Value::Null)
    }
}


pub mod builtin;