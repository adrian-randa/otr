use std::fmt::Display;
use serde::{Deserialize, Serialize};

use crate::{expression::Operator, procedure::CompiledProcedure, r#struct::Struct, vec_map::VecMap};

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

impl ImportAddress {
    pub fn to_flat_string(self) -> String {
        fn nibble_to_hex(nibble: u8) -> char {
            if nibble <= 9 {
                (b'0' + nibble) as char
            } else {
                (b'a' - 10 + nibble) as char
            }
        }

        fn byte_to_hex(byte: u8) -> [char; 2] {
            [
                nibble_to_hex(byte >> 4),
                nibble_to_hex(byte & 0b00001111)
            ]
        }

        let path_hex: String = self.path
            .map(|path| {
                path.as_bytes()
                    .iter()
                    .fold(Vec::new(), |mut acc, byte| {
                        acc.extend_from_slice(&byte_to_hex(*byte));
                        acc
                    })
                    .iter().collect()
            })
            .unwrap_or("".into());

        self.module_id + &path_hex
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompiledModule {
    dependencies: Vec<ImportAddress>,

    struct_prototypes: VecMap<String, (Struct, bool)>,
    procedures: VecMap<String, (Box<CompiledProcedure>, bool)>,

    associated_procedures: VecMap<String, VecMap<String, (Box<CompiledProcedure>, bool)>>,

    operator_overloads: VecMap<String, VecMap<Operator, (Box<CompiledProcedure>, bool)>>,
}

impl CompiledModule {
    pub fn insert_procedure(
        &mut self,
        identifier: String,
        procedure: Box<CompiledProcedure>,
        exported: bool,
    ) {
        self.procedures.insert(identifier.into(), (procedure, exported));
    }

    pub fn get_procedure(&self, identifier: &str) -> Option<&(Box<CompiledProcedure>, bool)> {
        self.procedures.get(identifier)
    }

    pub fn get_procedure_mut(&mut self, identifier: &str) -> Option<&mut (Box<CompiledProcedure>, bool)> {
        self.procedures.get_mut(identifier)
    }

    pub fn insert_associated_procedure(
        &mut self,
        struct_identifier: String,
        procedure_identifier: String,
        procedure: Box<CompiledProcedure>,
        exported: bool,
    ) {
        if let Some(lut) = self.associated_procedures.get_mut(&struct_identifier) {
            lut.insert(procedure_identifier, (procedure, exported));
        } else {
            let mut map = VecMap::new();
            map.insert(procedure_identifier, (procedure, exported));
    
            self.associated_procedures.insert(struct_identifier, map);
        }
    }

    pub fn get_associated_procedure(
        &self,
        struct_ident: &str,
        procedure_ident: &str,
    ) -> Option<&(Box<CompiledProcedure>, bool)> {
        self
            .associated_procedures
            .get(struct_ident)
            .and_then(|lut| lut.get(procedure_ident))
    }

    pub fn get_associated_procedure_mut(
        &mut self,
        struct_ident: &str,
        procedure_ident: &str,
    ) -> Option<&mut (Box<CompiledProcedure>, bool)> {
        self
            .associated_procedures
            .get_mut(struct_ident)
            .and_then(|lut| lut.get_mut(procedure_ident))
    }

    pub fn insert_operator(&mut self, struct_identifier: String, operator: Operator, procedure: Box<CompiledProcedure>, exported: bool) {
        if let Some(lut) = self.operator_overloads.get_mut(&struct_identifier) {
            lut.insert(operator, (procedure, exported));
        } else {
            let mut lut = VecMap::new();
            lut.insert(operator, (procedure, exported));
            self.operator_overloads.insert(struct_identifier, lut);
        }
    }

    pub fn get_operator(&self, struct_identifier: &str, operator: Operator) -> Option<&(Box<CompiledProcedure>, bool)> {
        self.operator_overloads.get(struct_identifier).and_then(|lut| lut.get(operator))
    }

    pub fn get_operator_mut(&mut self, struct_identifier: &str, operator: Operator) -> Option<&mut (Box<CompiledProcedure>, bool)> {
        self.operator_overloads.get_mut(struct_identifier).and_then(|lut| lut.get_mut(operator))
    }


    pub fn push_dependency(&mut self, dependency: ImportAddress) {
        self.dependencies.push(dependency);
    }

    pub fn get_dependencies(&self) -> &[ImportAddress] {
        &self.dependencies
    }

    

    pub fn insert_struct(&mut self, ident: String, prototype: Struct, exported: bool) {
        self.struct_prototypes
            .insert(ident, (prototype, exported));
    }

    pub fn get_struct(&self, ident: &str) -> Option<&(Struct, bool)> {
        self.struct_prototypes.get(ident)
    }

    pub fn get_struct_mut(&mut self, ident: &str) -> Option<&mut (Struct, bool)> {
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
    pub fn new(module_id: String, ident: String) -> Self {
        Self {
            module_id,
            ident,
        }
    }

    pub fn get_module_id(&self) -> &String {
        &self.module_id
    }

    pub fn get_identifier(&self) -> &String {
        &self.ident
    }
}