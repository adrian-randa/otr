use std::collections::HashMap;

use crate::{error::{Error, compiler_error::CompilerError}, runtime::{ModuleAddress, RuntimeError, Struct, environment::Environment, procedures::Procedure}};

use crate::error::Result;

#[derive(Debug, Default)]
pub struct Module {
    struct_prototypes: HashMap<String, (Struct, bool)>,
    procedures: HashMap<String, (Box<dyn Procedure>, bool)>,
    associated_procedures: HashMap<(String, String), (Box<dyn Procedure>, bool)>
}

impl Module {
    pub fn insert_procedure(&mut self, identifier: String, procedure: Box<dyn Procedure>, exported: bool) {
        self.procedures.insert(identifier, (procedure, exported));
    }

    pub fn get_procedure(&self, identifier: &String, private_access: bool) -> Result<&Box<dyn Procedure>> {
        match self.procedures.get(identifier) {
            Some((proc, exported)) => {
                if *exported || private_access {
                    Ok(proc)
                } else {
                    Err(RuntimeError::ProcedureNotExported { procedure_identifier: identifier.clone() }.boxed())
                }
            }
            None => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: identifier.clone() }.boxed())
        }
    }

    pub fn insert_associated_procedure(
        &mut self,
        struct_identifier: String,
        procedure_identifier: String,
        procedure: Box<dyn Procedure>,
        exported: bool
    ) {
        self.associated_procedures.insert((struct_identifier, procedure_identifier), (procedure, exported));
    }

    pub fn get_associated_procedure(
        &self,
        struct_identifier: &String,
        procedure_identifier: &String,
        private_access: bool
    ) -> Result<&Box<dyn Procedure>> {
        match self.associated_procedures.get(&(struct_identifier.clone(), procedure_identifier.clone())) {
            Some((proc, exported)) => {
                if *exported || private_access {
                    Ok(proc)
                } else {
                    let procedure_identifier = procedure_identifier.clone();
                    let struct_identifier = struct_identifier.clone();
                    Err(RuntimeError::AssociatedProcedureNotExported { procedure_identifier, struct_identifier }.boxed())
                }
            }
            None => Err(RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: procedure_identifier.clone(), struct_identifier: struct_identifier.clone() }.boxed())
        }
    }

    pub fn insert_struct(&mut self, identifier: String, prototype: Struct, exported: bool) {
        self.struct_prototypes.insert(identifier, (prototype, exported));
    }

    pub fn get_struct(&self, identifier: &String, private_access: bool) -> Result<Struct> {
        match self.struct_prototypes.get(identifier) {
            Some((prototype, exported)) => {
                if *exported || private_access {
                    Ok(prototype.clone())
                } else {
                    Err(RuntimeError::StructNotExported { struct_identifier: identifier.clone() }.boxed())
                }
            }
            None => Err(RuntimeError::StructNotDefined { struct_identifier: identifier.clone() }.boxed())
        }
    }

    pub fn set_procedure_visibility(&mut self, member_ident: &String, visibility: bool) -> Result<()> {
        if let Some(member) = self.procedures.get_mut(member_ident) {
            member.1 = visibility;
            return Ok(());
        }
        Err(CompilerError::NoSuchMember { member_identifier: member_ident.clone() }.boxed())
    }

    pub fn set_struct_visibility(&mut self, member_ident: &String, visibility: bool) -> Result<()> {
        if let Some(member) = self.struct_prototypes.get_mut(member_ident) {
            member.1 = visibility;
            return Ok(());
        }
        Err(CompilerError::NoSuchMember { member_identifier: member_ident.clone() }.boxed())
    }

    pub fn set_associated_precedure_visibility(
        &mut self,
        struct_ident: &String,
        member_ident: &String,
        visibility: bool
    ) -> Result<()> {
        if let Some(member) = self.associated_procedures.get_mut(
            &(struct_ident.to_owned(), member_ident.to_owned())
        ) {
            member.1 = visibility;
            return Ok(());
        }
        Err(CompilerError::NoSuchMember { member_identifier: format!("{struct_ident}->{member_ident}") }.boxed())
    }
}
