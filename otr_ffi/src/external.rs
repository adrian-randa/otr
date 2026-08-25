use otr_core::vec_map::VecMap;

#[derive(Debug)]
pub struct ExternalFunction {
    pub parameters: Vec<String>,
}

#[derive(Debug)]
pub struct ExternalModule {
    pub library_name: String,
    pub functions: VecMap<String, ExternalFunction>,
}