use serde::{Deserialize, Serialize};

use crate::core::expression::{Expression, variable::VariableAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    PushVarToScope {
        identifier: String,
    },
    PopVarFromScope {
        identifier: String,
    },
    GrowStack,
    ShrinkStack,
    EvaluateExpression {
        expression: Expression,
        target: Option<VariableAddress>,
    },
    JumpConditional {
        condition_expression: Expression,
        jump_target: usize,
    },
    Return {
        expression: Expression,
    },
    Throw {
        expression: Expression,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProcedure {
    argument_identifiers: Vec<String>,
    instructions: Vec<Instruction>,
}

impl CompiledProcedure {
    pub(crate) fn new(argument_identifiers: Vec<String>, instructions: Vec<Instruction>) -> Self {
        Self { argument_identifiers, instructions }
    }
    pub(crate) fn get_argument_identifiers(&self) -> &Vec<String> {
        &self.argument_identifiers
    }
    
    pub(crate) fn get_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
}