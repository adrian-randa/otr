use colored::{Color, Colorize};

#[derive(Debug)]
pub enum TokenizerError {}

impl super::Error for TokenizerError {}

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
    pub(crate) fn boxed(self) -> Box<dyn super::Error> {
        Box::new(self)
    }
}
