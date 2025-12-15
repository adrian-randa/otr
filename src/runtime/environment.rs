use super::ScopeAddress;

use super::Value;

use super::RuntimeError;

use crate::runtime::Struct;
use crate::runtime::module::Module;
use crate::runtime::procedures::Procedure;

use super::ModuleAddress;

use super::Scope;

use super::module;

use std::rc::Rc;

use std::collections::HashMap;

pub struct Environment {
    //TODO: Remove public visibility
    pub contained_module_id: String,
    pub loaded_modules: HashMap<String, Rc<Module>>,
    pub scope: Scope,
}

impl Environment {
    pub fn new(contained_module_id: String) -> Self {
        Self {
            contained_module_id,
            loaded_modules: Default::default(),
            scope: Default::default(),
        }
    }

    pub fn get_procedure_by_address(&self, address: &ModuleAddress) -> Result<&Box<dyn Procedure>, RuntimeError> {
        let module = self
            .loaded_modules
            .get(address.get_module_id())
            .ok_or(RuntimeError {
                message: format!(
                    "Module \"{}\" not loaded in this environment!",
                    address.get_module_id()
                ),
            })?;

        module.get_procedure(
            address.get_identifier(),
            address.get_module_id() == &self.contained_module_id,
        )
    }

    pub fn get_struct_by_address(&self, address: &ModuleAddress) -> Result<Struct, RuntimeError> {
        let module = self
            .loaded_modules
            .get(address.get_module_id())
            .ok_or(RuntimeError {
                message: format!(
                    "Module \"{}\" not loaded in this environment!",
                    address.get_module_id()
                ),
            })?;

        module.get_struct(
            address.get_identifier(),
            address.get_module_id() == &self.contained_module_id,
        )
    }

    pub fn open_subenvironment(&self, new_scope: Scope, module_address: &ModuleAddress) -> Self {
        Self {
            contained_module_id: module_address.module_id.clone(),
            loaded_modules: self.loaded_modules.clone(),
            scope: new_scope,
        }
    }

    pub fn insert_members(&mut self, members: HashMap<String, Value>) {
        self.scope.insert_members(members);
    }

    pub fn lookup_variable(&self, address: ScopeAddress) -> Result<Value, RuntimeError> {
        let address = address.try_bake(self)?;

        let value = self
            .scope
            .get_variable(address, &self.contained_module_id)?;

        Ok(value.clone())
    }

    pub fn set_variable(
        &mut self,
        address: ScopeAddress,
        new_value: Value,
    ) -> Result<(), RuntimeError> {
        let address = address.try_bake(self)?;

        let value = self
            .scope
            .get_variable_mut(address, &self.contained_module_id)?;

        *value = new_value;

        Ok(())
    }

    pub fn load_module(&mut self, module_identifier: String, module: Rc<Module>) {
        self.loaded_modules.insert(module_identifier, module);
    }

    pub fn get_contained_module_id(&self) -> &String {
        &self.contained_module_id
    }
}
