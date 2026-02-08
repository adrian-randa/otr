

pub trait Error: std::fmt::Display {}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait ErrorContextualizer {
    fn contextualize(&self, error: Box<dyn Error>) -> Box<dyn Error>;
}

pub(crate) mod context;
pub(crate) mod runtime_error;
pub(crate) mod fragmenter_error;
pub(crate) mod tokenizer_error;
pub(crate) mod compiler_error;