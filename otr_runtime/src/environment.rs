use super::scope::{Scope};

use otr_core::expression::variable::{VariableAddress, VariableAddressant};
use otr_core::r#struct::Struct;
use otr_core::value::Value;

use crate::module::{Module, RuntimeModule};
use crate::procedures::RuntimeProcedure;
use crate::error::RuntimeError;

use otr_core::module::ModuleAddress;

use otr_core::error::Result;
use crate::scope::{BakedVariableAddress, try_bake_variable_address};

use std::rc::Rc;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment<'a> {
    contained_module_id: String,
    loaded_modules: HashMap<String, Rc<RuntimeModule<'a>>>,
    scope: Scope,
}

impl Default for Environment<'_> {
    fn default() -> Self {
        Self {
            contained_module_id: Default::default(),
            loaded_modules: Default::default(),
            scope: Scope::new(0),
        }
    }
}

#[allow(unused)]
impl Environment<'_> {
    pub fn new(contained_module_id: String, stack_size: usize) -> Self {
        Self {
            contained_module_id,
            loaded_modules: Default::default(),
            scope: Scope::new(stack_size),
        }
    }

    pub(crate) fn get_loaded_module(&'_ self, module_id: &String) -> Option<&'_ RuntimeModule<'_>> {
        self.loaded_modules
            .get(module_id)
            .map(|module| module.as_ref())
    }

    pub(crate) fn get_procedure_by_address(&'_ self, address: &ModuleAddress) -> Result<RuntimeProcedure<'_>> {
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
            contained_module_id: module_address.get_module_id().clone(),
            loaded_modules: self.loaded_modules.clone(),
            scope: new_scope,
        }
    }

    pub fn query_variable(&self, address: VariableAddress) -> Result<Value> {
        let address = try_bake_variable_address(address, self)?;

        self.scope
            .query_variable(address, &self.contained_module_id)
    }

    pub fn set_variable(&mut self, address: VariableAddress, new_value: Value) -> Result<()> {
        let address = try_bake_variable_address(address, self)?;

        self.scope
            .set_variable(address, &self.contained_module_id, new_value)
    }

    pub fn reference_variable(&self, address: VariableAddress) -> Result<Value> {
        let address = try_bake_variable_address(address, self)?;

        self.scope
            .reference_variable(address, &self.contained_module_id)
    }

    pub fn clone_variable(&self, address: VariableAddress) -> Result<Value> {
        let address = try_bake_variable_address(address, self)?;

        self.scope
            .clone_variable(address, &self.contained_module_id)
    }

    pub fn get_variable_type(&self, address: VariableAddress) -> Result<Value> {
        let address = try_bake_variable_address(address, self)?;

        self.scope.query_type(address, &self.contained_module_id)
    }

    pub fn get_contained_module_id(&self) -> &String {
        &self.contained_module_id
    }

    pub fn _get_scope(&self) -> &Scope {
        &self.scope
    }

    pub fn get_scope_mut(&mut self) -> &mut Scope {
        &mut self.scope
    }
}

#[allow(private_interfaces)]
impl<'a> Environment<'a> {
    pub fn load_module(&mut self, module_identifier: String, module: Rc<RuntimeModule<'a>>) {
        match module.as_ref() {
            RuntimeModule::Abstract(_) => {
                self.loaded_modules.insert(module_identifier, module);
            },
            RuntimeModule::AbstractRef(_) => panic!("Can only insert owned runtime modules into environments!"),
            RuntimeModule::Compiled(_) => {
                self.loaded_modules.insert(module_identifier, module);
            },
            RuntimeModule::CompiledRef(_) => panic!("Can only insert owned runtime modules into environments!"),
        }
    }
}

pub mod environment_builder;
mod features;