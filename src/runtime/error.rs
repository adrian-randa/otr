use crate::{core::value::Value, error::Error};

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
        self.value.clone()
    }
}

impl ValueError {
    pub(crate) fn new(value: Value) -> Self {
        Self { value }
    }
}