use std::collections::HashMap;

use crate::{compiler::{CompilerError, Decorator}, lexer::token::Token, runtime::{ModuleAddress, RuntimeObject}};

pub struct EntrypointDecorator {
    procedure_id: ModuleAddress
}

impl EntrypointDecorator {
    pub fn new(procedure_id: ModuleAddress) -> Self {
        Self { procedure_id }
    }
}

impl Decorator for EntrypointDecorator {
    fn apply(self: Box<Self>, runtime_object: &mut RuntimeObject) -> Result<(), CompilerError> {
        if runtime_object.entrypoint.is_some() {
            Err(CompilerError {
                message: format!("Duplicate entrypoint! Entrypoint is already set to {:?}!", runtime_object.entrypoint)
            })
        } else {
            runtime_object.entrypoint = Some(self.procedure_id);
            Ok(())
        }
    }
}