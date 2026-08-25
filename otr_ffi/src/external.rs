use otr_core::vec_map::VecMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalFunction {
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalModule {
    pub library_name: String,
    pub functions: VecMap<String, ExternalFunction>,
}