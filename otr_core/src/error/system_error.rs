use colored::Colorize;

use crate::{error::Error, value::Value};

#[derive(Debug)]
pub struct SystemError {
    message: String,
}

impl SystemError {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}

impl Error for SystemError {
    fn to_value(&self) -> Value {
        panic!("System Errors cannot be turned into values!")
    }
}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            "System Error!".on_red(),
            (&self.message as &str).red()
        )
    }
}