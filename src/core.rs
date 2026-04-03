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
    root: Option<String>,
}

impl IntoIterator for CompiledObject {
    type Item = (String, CompiledModule);

    type IntoIter = hash_map::IntoIter<String, CompiledModule>;

    fn into_iter(self) -> Self::IntoIter {
        self.modules.into_iter()
    }
}

impl CompiledObject {
    pub(crate) fn new(root: String) -> Self {
        Self { modules: HashMap::new(), root: Some(root) }
    }

    pub(crate) fn insert_module(&mut self, identifier: String, module: CompiledModule) {
        self.modules.insert(identifier, module);
    }

    pub(crate) fn get_root(&self) -> &Option<String> {
        &self.root
    }

    pub(crate) fn root(&mut self) -> Option<String> {
        self.root.take()
    }

    pub(crate) fn set_root(&mut self, root: String) {
        self.root = Some(root);
    }
}