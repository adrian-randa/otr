use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{core::{procedure::CompiledProcedure, r#struct::Struct}};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ImportAddress {
    pub module_id: String,
    pub path: Option<String>,
}

impl Display for ImportAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            self.path.as_ref().unwrap_or(&("".to_string())),
            self.module_id
        )
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompiledModule {
    dependencies: Vec<ImportAddress>,

    struct_prototypes: HashMap<String, (Struct, bool)>,
    procedures: HashMap<String, (Box<CompiledProcedure>, bool)>,
    associated_procedures: HashMap<(String, String), (Box<CompiledProcedure>, bool)>,
}

impl CompiledModule {
    pub(crate) fn insert_procedure(
        &mut self,
        identifier: String,
        procedure: Box<CompiledProcedure>,
        exported: bool,
    ) {
        self.procedures.insert(identifier, (procedure, exported));
    }

    pub(crate) fn get_procedure(&self, identifier: &str) -> Option<&(Box<CompiledProcedure>, bool)> {
        self.procedures.get(identifier)
    }

    pub(crate) fn get_procedure_mut(&mut self, identifier: &str) -> Option<&mut (Box<CompiledProcedure>, bool)> {
        self.procedures.get_mut(identifier)
    }

    pub(crate) fn insert_associated_procedure(
        &mut self,
        struct_identifier: String,
        procedure_identifier: String,
        procedure: Box<CompiledProcedure>,
        exported: bool,
    ) {
        self.associated_procedures.insert(
            (struct_identifier, procedure_identifier),
            (procedure, exported),
        );
    }

    pub(crate) fn get_associated_procedure(
        &self,
        struct_ident: &str,
        procedure_ident: &str,
    ) -> Option<&(Box<CompiledProcedure>, bool)> {
        self
            .associated_procedures
            .get(&(struct_ident.to_string(), procedure_ident.to_string()))
    }

    pub(crate) fn get_associated_procedure_mut(
        &mut self,
        struct_ident: &str,
        procedure_ident: &str,
    ) -> Option<&mut (Box<CompiledProcedure>, bool)> {
        self
            .associated_procedures
            .get_mut(&(struct_ident.to_string(), procedure_ident.to_string()))
    }


    pub(crate) fn push_dependency(&mut self, dependency: ImportAddress) {
        self.dependencies.push(dependency);
    }

    pub(crate) fn get_dependencies(&self) -> &[ImportAddress] {
        &self.dependencies
    }

    

    pub(crate) fn insert_struct(&mut self, ident: String, prototype: Struct, exported: bool) {
        self.struct_prototypes
            .insert(ident, (prototype, exported));
    }

    pub(crate) fn get_struct(&self, ident: &str) -> Option<&(Struct, bool)> {
        self.struct_prototypes.get(ident)
    }

    pub(crate) fn get_struct_mut(&mut self, ident: &str) -> Option<&mut (Struct, bool)> {
        self.struct_prototypes.get_mut(ident)
    }
}



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleAddress {
    module_id: String,
    ident: String,
}

impl From<(&str, &str)> for ModuleAddress {
    fn from(value: (&str, &str)) -> Self {
        Self {
            module_id: value.0.to_string(),
            ident: value.1.to_string(),
        }
    }
}

impl std::fmt::Display for ModuleAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.module_id, self.ident)
    }
}

impl ModuleAddress {
    pub(crate) fn new(module_id: String, ident: String) -> Self {
        Self {
            module_id,
            ident,
        }
    }

    pub(crate) fn get_module_id(&self) -> &String {
        &self.module_id
    }

    pub(crate) fn get_identifier(&self) -> &String {
        &self.ident
    }
}