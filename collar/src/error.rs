use otr_core::Error;
use colored::Colorize;

#[derive(Debug)]
pub struct CollarError {
    message: String,
}

impl CollarError {
    pub fn new(message: impl ToString) -> Self {
        Self { message: message.to_string() }
    }

    pub fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    } 
}

impl std::fmt::Display for CollarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            "Error!".on_red(),
            self.message,
        )
    }
}

impl Error for CollarError {
    fn to_value(&self) -> otr_core::value::Value {
        panic!("Collar Errors cannot be turend into values!")
    }
}