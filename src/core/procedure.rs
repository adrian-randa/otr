use crate::error::Result;

pub trait Procedure: std::fmt::Debug {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<Value>;
}


#[derive(Debug)]
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
        expression: Box<dyn Expression>,
        target: Option<ScopeAddress>,
    },
    JumpConditional {
        condition_expression: Box<dyn Expression>,
        jump_target: usize,
    },
    Return {
        expression: Box<dyn Expression>,
    },
    Throw {
        expression: Box<dyn Expression>,
    }
}

#[derive(Debug)]
pub struct CompiledProcedure {
    arguments_identifiers: Vec<String>,
    instructions: Vec<Instruction>,
}