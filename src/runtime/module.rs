use std::collections::HashMap;

use crate::{compiler::CompilerError, runtime::{ModuleAddress, RuntimeError, Struct, environment::Environment, procedures::Procedure}};

#[derive(Debug, Default)]
pub struct Module {
    struct_prototypes: HashMap<String, (Struct, bool)>,
    procedures: HashMap<String, (Box<dyn Procedure>, bool)>,
}

impl Module {
    pub fn insert_procedure(&mut self, identifier: String, procedure: Box<dyn Procedure>, exported: bool) {
        self.procedures.insert(identifier, (procedure, exported));
    }

    pub fn get_procedure(&self, identifier: &String, private_access: bool) -> Result<&Box<dyn Procedure>, RuntimeError> {
        match self.procedures.get(identifier) {
            Some((proc, exported)) => {
                if *exported || private_access {
                    Ok(proc)
                } else {
                    Err(RuntimeError {
                        message: format!(
                            "Procedure \"{}\" is not exported by this module!",
                            identifier
                        ),
                    })
                }
            }
            None => Err(RuntimeError {
                message: format!("Procedure \"{}\" not defined in this module!", identifier),
            })
        }
    }

    pub fn insert_struct(&mut self, identifier: String, prototype: Struct, exported: bool) {
        self.struct_prototypes.insert(identifier, (prototype, exported));
    }

    pub fn get_struct(&self, identifier: &String, private_access: bool) -> Result<Struct, RuntimeError> {
        match self.struct_prototypes.get(identifier) {
            Some((prototype, exported)) => {
                if *exported || private_access {
                    Ok(prototype.clone())
                } else {
                    Err(RuntimeError {
                        message: format!(
                            "Struct \"{}\" is not exported by this module!",
                            identifier
                        ),
                    })
                }
            }
            None => Err(RuntimeError {
                message: format!("Struct \"{}\" not defined in this module!", identifier),
            })
        }
    }

    pub fn set_member_visibility(&mut self, member_ident: &String, visibility: bool) -> Result<(), CompilerError> {

        if let Some(member) = self.procedures.get_mut(member_ident) {
            member.1 = visibility;
            return Ok(());
        }
        if let Some(member) = self.struct_prototypes.get_mut(member_ident) {
            member.1 = visibility;
            return Ok(());
        }

        Err(CompilerError {
            message: format!("Member '{}' not found!", member_ident)
        })
    }
}
