use crate::error::Result;

pub(crate) trait Module {
    fn insert_procedure(
        &mut self,
        identifier: String,
        procedure: Box<dyn Procedure>,
        exported: bool,
    );
    fn get_procedure(
        &self,
        identifier: &String,
        private_access: bool,
    ) -> Result<&Box<dyn Procedure>>;
    fn insert_associated_procedure(
        &mut self,
        struct_identifier: String,
        procedure_identifier: String,
        procedure: Box<dyn Procedure>,
        exported: bool,
    );
    fn get_associated_procedure(
        &self,
        struct_identifier: &String,
        procedure_identifier: &String,
        private_access: bool,
    ) -> Result<&Box<dyn Procedure>>;
    fn insert_struct(&mut self, identifier: String, prototype: Struct, exported: bool);
    fn get_struct(&self, identifier: &String, private_access: bool) -> Result<Struct>;
    fn set_procedure_visibility(&mut self, member_ident: &String, visibility: bool) -> Result<()>;
    fn set_struct_visibility(&mut self, member_ident: &String, visibility: bool) -> Result<()>;
    fn set_associated_precedure_visibility(
        &mut self,
        struct_ident: &String,
        member_ident: &String,
        visibility: bool,
    ) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct CompiledModule {
    struct_prototypes: HashMap<String, (Struct, bool)>,
    procedures: HashMap<String, (Box<dyn Procedure>, bool)>,
    associated_procedures: HashMap<(String, String), (Box<dyn Procedure>, bool)>,
}

impl Module for CompiledModule {
    fn insert_procedure(
        &mut self,
        identifier: String,
        procedure: Box<dyn Procedure>,
        exported: bool,
    ) {
        self.procedures.insert(identifier, (procedure, exported));
    }

    fn get_procedure(
        &self,
        identifier: &String,
        private_access: bool,
    ) -> Result<&Box<dyn Procedure>> {
        match self.procedures.get(identifier) {
            Some((proc, exported)) => {
                if *exported || private_access {
                    Ok(proc)
                } else {
                    Err(RuntimeError::ProcedureNotExported {
                        procedure_identifier: identifier.clone(),
                    }
                    .boxed())
                }
            }
            None => Err(RuntimeError::ProcedureNotDefined {
                procedure_identifier: identifier.clone(),
            }
            .boxed()),
        }
    }

    fn insert_associated_procedure(
        &mut self,
        struct_identifier: String,
        procedure_identifier: String,
        procedure: Box<dyn Procedure>,
        exported: bool,
    ) {
        self.associated_procedures.insert(
            (struct_identifier, procedure_identifier),
            (procedure, exported),
        );
    }

    fn get_associated_procedure(
        &self,
        struct_identifier: &String,
        procedure_identifier: &String,
        private_access: bool,
    ) -> Result<&Box<dyn Procedure>> {
        match self
            .associated_procedures
            .get(&(struct_identifier.clone(), procedure_identifier.clone()))
        {
            Some((proc, exported)) => {
                if *exported || private_access {
                    Ok(proc)
                } else {
                    let procedure_identifier = procedure_identifier.clone();
                    let struct_identifier = struct_identifier.clone();
                    Err(RuntimeError::AssociatedProcedureNotExported {
                        procedure_identifier,
                        struct_identifier,
                    }
                    .boxed())
                }
            }
            None => Err(RuntimeError::AssociatedProcedureNotDefined {
                procedure_identifier: procedure_identifier.clone(),
                struct_identifier: struct_identifier.clone(),
            }
            .boxed()),
        }
    }

    fn insert_struct(&mut self, identifier: String, prototype: Struct, exported: bool) {
        self.struct_prototypes
            .insert(identifier, (prototype, exported));
    }

    fn get_struct(&self, identifier: &String, private_access: bool) -> Result<Struct> {
        match self.struct_prototypes.get(identifier) {
            Some((prototype, exported)) => {
                if *exported || private_access {
                    Ok(prototype.clone())
                } else {
                    Err(RuntimeError::StructNotExported {
                        struct_identifier: identifier.clone(),
                    }
                    .boxed())
                }
            }
            None => Err(RuntimeError::StructNotDefined {
                struct_identifier: identifier.clone(),
            }
            .boxed()),
        }
    }

    fn set_procedure_visibility(&mut self, member_ident: &String, visibility: bool) -> Result<()> {
        if let Some(member) = self.procedures.get_mut(member_ident) {
            member.1 = visibility;
            return Ok(());
        }
        Err(CompilerError::NoSuchMember {
            member_identifier: member_ident.clone(),
        }
        .boxed())
    }

    fn set_struct_visibility(&mut self, member_ident: &String, visibility: bool) -> Result<()> {
        if let Some(member) = self.struct_prototypes.get_mut(member_ident) {
            member.1 = visibility;
            return Ok(());
        }
        Err(CompilerError::NoSuchMember {
            member_identifier: member_ident.clone(),
        }
        .boxed())
    }

    fn set_associated_precedure_visibility(
        &mut self,
        struct_ident: &String,
        member_ident: &String,
        visibility: bool,
    ) -> Result<()> {
        if let Some(member) = self
            .associated_procedures
            .get_mut(&(struct_ident.to_owned(), member_ident.to_owned()))
        {
            member.1 = visibility;
            return Ok(());
        }
        Err(CompilerError::NoSuchMember {
            member_identifier: format!("{struct_ident}->{member_ident}"),
        }
        .boxed())
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct ModuleAddress {
    module_id: String,
    identifier: String,
}

impl From<(&str, &str)> for ModuleAddress {
    fn from(value: (&str, &str)) -> Self {
        Self {
            module_id: value.0.to_string(),
            identifier: value.1.to_string(),
        }
    }
}

impl Display for ModuleAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.module_id, self.identifier)
    }
}

impl ModuleAddress {
    pub(crate) fn new(module_id: String, identifier: String) -> Self {
        Self {
            module_id,
            identifier,
        }
    }

    pub(crate) fn get_module_id(&self) -> &String {
        &self.module_id
    }

    pub(crate) fn get_identifier(&self) -> &String {
        &self.identifier
    }
}