use colored::Colorize;

use crate::error::Error;

#[derive(Debug)]
pub enum FragmentationError {
    InvalidControlCharacter {
        line_index: usize,
        column_index: usize,
    },
    LinebreakInStringLiteral {
        line_index: usize,
        column_index: usize,
    },
}

impl Error for FragmentationError {}

impl std::fmt::Display for FragmentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FragmentationError::InvalidControlCharacter {
                line_index,
                column_index,
            } => write!(
                f,
                "{} Invalid control character at line {line_index} column {column_index}!",
                "Fragmentation Error!".on_red()
            ),
            FragmentationError::LinebreakInStringLiteral {
                line_index,
                column_index,
            } => write!(
                f,
                "{} Newline in string literal at line {line_index} column {column_index}!",
                "Fragmentation Error!".on_red()
            ),
        }
    }
}

impl FragmentationError {
    pub fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}
