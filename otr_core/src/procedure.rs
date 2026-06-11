use serde::{Deserialize, Serialize};

use crate::expression::{Expression, variable::VariableAddress};

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
    pub instructions: Vec<Instruction>,
    pub num_args: usize,
    pub stack_size: usize,
}