use crate::{
    compiler::{CompilerError, Decorator}, core::{CompiledObject, module::ModuleAddress},
};

use crate::error::Result;

pub struct EntrypointDecorator {
    procedure_id: ModuleAddress,
}

impl EntrypointDecorator {
    pub fn new(procedure_id: ModuleAddress) -> Self {
        Self { procedure_id }
    }
}

impl Decorator for EntrypointDecorator {
    fn apply(self: Box<Self>, object: &mut CompiledObject) -> Result<()> {
        if object.get_entrypoint().is_some() {
            Err(CompilerError::Unknown {
                message: format!(
                    "Duplicate entrypoint! Entrypoint is already set to {:?}!",
                    object.get_entrypoint().as_ref().unwrap()
                ),
            }
            .boxed())
        } else {
            object.set_entrypoint(self.procedure_id);
            Ok(())
        }
    }
}
