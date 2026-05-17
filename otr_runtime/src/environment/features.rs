use crate::module::RuntimeModule;
use otr_core::error::Result;

pub(crate) trait FeatureBuilder {
    fn add_arg(&mut self, arg_ident: &dyn AsRef<str>, arg_value: &dyn AsRef<str>) -> Result<()>;

    fn build(&mut self) -> Result<RuntimeModule<'static>>; 
}

pub mod arrays;
pub mod debug;
pub mod files;
pub mod numbers;
pub mod strings;
pub mod math;