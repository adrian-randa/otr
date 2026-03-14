use crate::{error::Error};
use crate::core::value::Value;


#[derive(Debug)]
pub(crate) struct ValueError {
    value: Value,
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.to_string())
    }
}

impl Error for ValueError {
    fn to_value(&self) -> Value {
        self.value.get_value(()).unwrap()
    }
}

impl ValueError {
    pub(crate) fn new(value: Value) -> Self {
        Self { value }
    }
}