use crate::core::module::CompiledModule;
use crate::core::r#struct::Struct;
use crate::runtime::{error::RuntimeError, procedures::RuntimeProcedure};
use crate::error::Result;

pub(crate) trait Module: std::fmt::Debug {
    fn get_procedure(
        &'_ self,
        identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure<'_>>;
    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &String,
        procedure_identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure<'_>>;
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
        &'_ self,
        identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure<'_>> {
        match self {
            RuntimeModule::Abstract(module) => module.get_procedure(identifier, private_access),
            RuntimeModule::Compiled(compiled_module) => {
                let (procedure, public) = compiled_module.get_procedure(identifier)
                    .ok_or(RuntimeError::NoSuchMember { member_identifier: identifier.to_string() }.boxed())?;

                if !public && private_access {
                    Err(RuntimeError::FieldIsPrivate.boxed())
                } else {
                    Ok(RuntimeProcedure::CompiledRef(
                        procedure
                    ))
                }
            },
            RuntimeModule::AbstractRef(module) => module.get_procedure(identifier, private_access),
            RuntimeModule::CompiledRef(compiled_module) => {
                let (procedure, public) = compiled_module.get_procedure(identifier)
                    .ok_or(RuntimeError::ProcedureNotDefined { procedure_identifier: identifier.to_string() }.boxed())?;
                
                if !public && private_access {
                    Err(RuntimeError::ProcedureNotExported { procedure_identifier: identifier.clone() }.boxed())
                } else {
                    Ok(RuntimeProcedure::CompiledRef(
                        procedure
                    ))
                }
            },
        }
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &String,
        procedure_identifier: &String,
        private_access: bool,
    ) -> Result<RuntimeProcedure<'_>> {
        match self {
            RuntimeModule::Abstract(module) => module.get_associated_procedure(struct_identifier, procedure_identifier, private_access),
            RuntimeModule::Compiled(compiled_module) => {
                let (procedure, public) = compiled_module.get_associated_procedure(struct_identifier, procedure_identifier)
                    .ok_or(RuntimeError::AssociatedProcedureNotDefined {
                        procedure_identifier: procedure_identifier.to_string(),
                        struct_identifier: struct_identifier.to_string()
                    }.boxed())?;
                
                if !public && private_access {
                    Err(RuntimeError::AssociatedProcedureNotExported {
                        procedure_identifier: procedure_identifier.to_string(),
                        struct_identifier: struct_identifier.to_string()
                    }.boxed())
                } else {
                    Ok(RuntimeProcedure::CompiledRef(procedure))
                }
            },
            RuntimeModule::AbstractRef(module) => module.get_associated_procedure(struct_identifier, procedure_identifier, private_access),
            RuntimeModule::CompiledRef(compiled_module) => {
                let (procedure, public) = compiled_module.get_associated_procedure(struct_identifier, procedure_identifier)
                    .ok_or(RuntimeError::AssociatedProcedureNotDefined {
                        procedure_identifier: procedure_identifier.to_string(),
                        struct_identifier: struct_identifier.to_string()
                    }.boxed())?;

                if !public && private_access {
                    Err(RuntimeError::AssociatedProcedureNotExported {
                        procedure_identifier: procedure_identifier.to_string(),
                        struct_identifier: struct_identifier.to_string()
                    }.boxed())
                } else {
                    Ok(RuntimeProcedure::CompiledRef(procedure))
                }
            },
        }
    }

    fn get_struct(&self, identifier: &String, private_access: bool) -> Result<Struct> {
        match self {
            RuntimeModule::Abstract(module) => module.get_struct(identifier, private_access),
            RuntimeModule::Compiled(compiled_module) => {
                let (st, public) = compiled_module.get_struct(identifier)
                    .ok_or(RuntimeError::StructNotDefined { struct_identifier: identifier.to_string() }.boxed())?;

                if !public && private_access {
                    Err(RuntimeError::StructNotExported { struct_identifier: identifier.to_string() }.boxed())
                } else {
                    Ok(st.clone())
                }
            },
            RuntimeModule::AbstractRef(module) => module.get_struct(identifier, private_access),
            RuntimeModule::CompiledRef(compiled_module) => {
                let (st, public) = compiled_module.get_struct(identifier)
                    .ok_or(RuntimeError::StructNotDefined { struct_identifier: identifier.to_string() }.boxed())?;

                if !public && private_access {
                    Err(RuntimeError::StructNotExported { struct_identifier: identifier.to_string() }.boxed())
                } else {
                    Ok(st.clone())
                }
            },
        }
    }
}