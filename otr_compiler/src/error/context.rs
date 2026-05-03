use colored::Colorize;

use otr_core::{Error, value::Value};

#[derive(Debug)]
pub(crate) struct LineIndexContextDecorator {
    pub(crate) error: Box<dyn Error>,

    pub(crate) line: usize,
}

impl Error for LineIndexContextDecorator {
    fn to_value(&self) -> Value {
        self.error.to_value()
    }
}

impl std::fmt::Display for LineIndexContextDecorator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = format!(
            "Occurred on line {}.",
            self.line
        );

        write!(f, "{}\n{}", self.error, (&message as &str).bright_black())
    }
}

impl LineIndexContextDecorator {
    pub(crate) fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}