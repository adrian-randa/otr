use colored::{Color, Colorize};
use crate::core::value::Value;

#[allow(unused)]
#[derive(Debug)]
pub enum TokenizerError {}

impl super::Error for TokenizerError {
    fn to_value(&self) -> Value {
        panic!("Tokenizer Errors cannot be turned into values!")
    }
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &"Error while tokenizing source file!".color(Color::Red)
        )
    }
}

impl TokenizerError {
    pub(crate) fn _boxed(self) -> Box<dyn super::Error> {
        Box::new(self)
    }
}
