use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Weak;
use std::vec::IntoIter;
use std::{collections::HashMap, rc::Rc};

use itertools::Itertools;

use crate::error::compiler_error::CompilerError;
use crate::error::context::{HintContextDecorator, VariableContextDecorator};
use crate::error::runtime_error::RuntimeError;
use crate::error::Error;
use crate::lexer::token::{
    LiteralToken, PrimitiveTypeToken,
};
use crate::runtime::environment::Environment;
use crate::runtime::scope::ScopeAddressant;

use crate::error::Result;

pub mod environment;
pub mod expressions;
pub mod module;
pub mod procedures;

pub trait Expression: std::fmt::Debug {
    fn eval(&self, environment: &Environment) -> Result<Value>;
}


impl RuntimeObject {
    pub(crate) fn new() -> Self {
        Self {
            base_environement: Environment::new("".into()),
            entrypoint: None,
        }
    }

    pub fn execute(self) -> Result<Value> {
        let entrypoint = self.entrypoint.ok_or(
            HintContextDecorator {
                error: RuntimeError::NoEntrypoint.boxed(),
                message: "If you want to run the specified file as a script, please annotate a procedure as the entrypoint.".into()
            }.boxed()
        )?;

        let main_expression = ProcedureCallExpression::new(entrypoint, Vec::new());

        main_expression.eval(&self.base_environement)
    }
}

pub mod scope;
