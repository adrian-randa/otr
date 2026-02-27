use super::scope::{Scope, ScopeAddress};

use super::Value;

use super::RuntimeError;

use crate::error::Error;
use crate::runtime::module::{CompiledModule, Module};
use crate::runtime::procedures::builtin::{arrays, debug, files, numbers, strings};
use crate::runtime::procedures::Procedure;
use crate::runtime::Struct;

use super::ModuleAddress;

use crate::error::Result;

use std::rc::Rc;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    //TODO: Remove public visibility
    pub contained_module_id: String,
    pub loaded_modules: HashMap<String, Rc<CompiledModule>>,
    pub scope: Scope,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            contained_module_id: Default::default(),
            loaded_modules: HashMap::from_iter(
                vec![
                    ("Arrays".into(), Rc::new(arrays::get_module())),
                    ("Strings".into(), Rc::new(strings::get_module())),
                    ("Numbers".into(), Rc::new(numbers::get_module())),
                    ("Debug".into(), Rc::new(debug::get_module())),
                    ("Files".into(), Rc::new(files::get_module())),
                ]
                .into_iter(),
            ),
            scope: Default::default(),
        }
    }
}

impl Environment {
    pub fn new(contained_module_id: String) -> Self {
        Self {
            contained_module_id,
            loaded_modules: Default::default(),
            scope: Default::default(),
        }
    }

    pub fn get_loaded_module(&self, module_id: &String) -> Option<&CompiledModule> {
        self.loaded_modules
            .get(module_id)
            .map(|module| module.as_ref())
    }

    pub fn get_procedure_by_address(&self, address: &ModuleAddress) -> Result<&Box<dyn Procedure>> {
        let module = self.loaded_modules.get(address.get_module_id()).ok_or(
            RuntimeError::ModuleNotLoaded {
                module_identifier: address.get_module_id().clone(),
            }
            .boxed(),
        )?;

        module.get_procedure(
            address.get_identifier(),
            address.get_module_id() == &self.contained_module_id,
        )
    }

    pub fn get_struct_by_address(&self, address: &ModuleAddress) -> Result<Struct> {
        let module = self.loaded_modules.get(address.get_module_id()).ok_or(
            RuntimeError::ModuleNotLoaded {
                module_identifier: address.get_module_id().clone(),
            }
            .boxed(),
        )?;

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

    pub fn query_variable(&self, address: ScopeAddress) -> Result<Value> {
        let address = address.try_bake(self)?;

        self.scope
            .query_variable(address, &self.contained_module_id)
    }

    pub fn set_variable(&mut self, address: ScopeAddress, new_value: Value) -> Result<()> {
        let address = address.try_bake(self)?;

        self.scope
            .set_variable(address, &self.contained_module_id, new_value)
    }

    pub fn reference_variable(&self, address: ScopeAddress) -> Result<Value> {
        let address = address.try_bake(self)?;

        self.scope
            .reference_variable(address, &self.contained_module_id)
    }

    pub(crate) fn clone_variable(&self, address: ScopeAddress) -> Result<Value> {
        let address = address.try_bake(self)?;

        self.scope
            .clone_variable(address, &self.contained_module_id)
    }

    pub(crate) fn get_variable_type(&self, address: ScopeAddress) -> Result<Value> {
        let address = address.try_bake(self)?;

        self.scope.query_type(address, &self.contained_module_id)
    }

    pub fn load_module(&mut self, module_identifier: String, module: Rc<CompiledModule>) {
        self.loaded_modules.insert(module_identifier, module);
    }

    pub fn get_contained_module_id(&self) -> &String {
        &self.contained_module_id
    }
}
