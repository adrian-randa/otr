use crate::value::Value;

pub trait Error: std::fmt::Display + std::fmt::Debug {
    fn to_value(&self) -> Value;
}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait ErrorContextualizer {
    fn contextualize(&self, error: Box<dyn Error>) -> Box<dyn Error>;
}

pub mod system_error;