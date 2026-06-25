use std::collections::{HashMap, hash_map};

use module::{CompiledModule};

pub mod value;
pub mod member;
pub mod r#type;
pub mod r#struct;
pub mod procedure;
pub mod module;
pub mod expression;
pub mod vec_map;
pub mod error;

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
    pub fn new(root: String) -> Self {
        Self { modules: HashMap::new(), root: Some(root) }
    }

    pub fn insert_module(&mut self, identifier: String, module: CompiledModule) {
        self.modules.insert(identifier, module);
    }

    pub fn get_root(&self) -> &Option<String> {
        &self.root
    }

    pub fn root(&mut self) -> Option<String> {
        self.root.take()
    }

    pub fn set_root(&mut self, root: String) {
        self.root = Some(root);
    }
}

pub use error::{Result, Error, system_error::SystemError};