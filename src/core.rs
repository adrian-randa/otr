use std::collections::{HashMap, hash_map};

use crate::core::module::{CompiledModule, ModuleAddress};

pub mod value;
pub mod member;
pub mod r#type;
pub mod r#struct;
pub mod procedure;
pub mod module;
pub mod expression;

#[derive(Debug, Clone)]
pub struct CompiledObject {
    modules: HashMap<String, CompiledModule>,
    entrypoint: Option<ModuleAddress>,
}

impl IntoIterator for CompiledObject {
    type Item = (String, CompiledModule);

    type IntoIter = hash_map::IntoIter<String, CompiledModule>;

    fn into_iter(self) -> Self::IntoIter {
        self.modules.into_iter()
    }
}

impl CompiledObject {
    pub(crate) fn new() -> Self {
        Self { modules: HashMap::new(), entrypoint: None }
    }

    pub(crate) fn insert_module(&mut self, identifier: String, module: CompiledModule) {
        self.modules.insert(identifier, module);
    }

    pub(crate) fn get_entrypoint(&self) -> &Option<ModuleAddress> {
        &self.entrypoint
    }

    pub(crate) fn entrypoint(&mut self) -> Option<ModuleAddress> {
        self.entrypoint.take()
    }

    pub(crate) fn set_entrypoint(&mut self, entrypoint: ModuleAddress) {
        self.entrypoint = Some(entrypoint);
    }
}