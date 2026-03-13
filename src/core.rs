pub mod value;
pub mod member;
pub mod r#type;
pub mod r#struct;
pub mod procedure;
pub mod module;
pub mod expression;

#[derive(Debug)]
pub struct RuntimeObject {
    pub(crate) base_environement: Environment,
    pub(crate) entrypoint: Option<ModuleAddress>,
}