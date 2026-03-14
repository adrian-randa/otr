use crate::core::module::CompiledModule;
use crate::core::r#struct::Struct;
use crate::runtime::procedures::RuntimeProcedure;
use crate::error::Result;

pub(crate) trait Module: std::fmt::Debug {
    fn get_procedure(
        &self,
        identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure>;
    fn get_associated_procedure(
        &self,
        struct_identifier: &String,
        procedure_identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure>;
    fn get_struct(&self, identifier: &String, private_access: bool) -> Result<Struct>;
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum RuntimeModule<'a> {
    Abstract(Box<dyn Module>),
    AbstractRef(&'a dyn Module),
    Compiled(CompiledModule),
    CompiledRef(&'a CompiledModule),
}

impl<'a> Module for RuntimeModule<'a> {
    fn get_procedure(
        &self,
        identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure> {
        match self {
            RuntimeModule::Abstract(module) => module.get_procedure(identifier, private_access),
            RuntimeModule::Compiled(compiled_module) => Ok(RuntimeProcedure::CompiledRef(
                compiled_module.get_procedure(identifier, private_access)?
            )),
            RuntimeModule::AbstractRef(module) => module.get_procedure(identifier, private_access),
            RuntimeModule::CompiledRef(compiled_module) => Ok(RuntimeProcedure::CompiledRef(
                compiled_module.get_procedure(identifier, private_access)?
            )),
        }
    }

    fn get_associated_procedure(
        &self,
        struct_identifier: &String,
        procedure_identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure> {
        match self {
            RuntimeModule::Abstract(module) => module.get_associated_procedure(struct_identifier, procedure_identifier, private_access),
            RuntimeModule::Compiled(compiled_module) => Ok(RuntimeProcedure::CompiledRef(compiled_module.get_associated_procedure(struct_identifier, procedure_identifier, private_access)?)),
            RuntimeModule::AbstractRef(module) => module.get_associated_procedure(struct_identifier, procedure_identifier, private_access),
            RuntimeModule::CompiledRef(compiled_module) => Ok(RuntimeProcedure::CompiledRef(
                compiled_module.get_associated_procedure(struct_identifier, procedure_identifier, private_access)?
            )),
        }
    }

    fn get_struct(&self, identifier: &String, private_access: bool) -> Result<Struct> {
        match self {
            RuntimeModule::Abstract(module) => module.get_struct(identifier, private_access),
            RuntimeModule::Compiled(compiled_module) => compiled_module.get_struct(identifier, private_access),
            RuntimeModule::AbstractRef(module) => module.get_struct(identifier, private_access),
            RuntimeModule::CompiledRef(compiled_module) => compiled_module.get_struct(identifier, private_access),
        }
    }
}