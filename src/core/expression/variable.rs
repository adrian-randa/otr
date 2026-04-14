use derive_more::{Deref, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::core::expression::Expression;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableExpression {
    address: VariableAddress,
    access_mode: VariableAccessMode,
}

impl VariableExpression {
    pub(crate) fn new(address: VariableAddress, access_mode: VariableAccessMode) -> Self {
        Self { address, access_mode }
    }

    pub(crate) fn get_address(&self) -> &VariableAddress {
        &self.address
    }

    pub(crate) fn get_access_mode(&self) -> VariableAccessMode {
        self.access_mode
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum VariableAccessMode {
    Move,
    Clone,
    Ref,
    TypeOf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableAddressant {
    Identifier(String),
    Index(usize),
    DynamicIndex(Expression),
}

impl From<&str> for VariableAddressant {
    fn from(value: &str) -> Self {
        Self::Identifier(value.into())
    }
}

impl From<usize> for VariableAddressant {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

#[derive(Debug, Clone, Deref, IntoIterator, Serialize, Deserialize)]
pub struct VariableAddress(Vec<VariableAddressant>);

impl TryFrom<Vec<VariableAddressant>> for VariableAddress {
    type Error = ();

    fn try_from(value: Vec<VariableAddressant>) -> std::result::Result<Self, Self::Error> {
        if value.is_empty() {
            Err(())
        } else {
            Ok(Self(value))
        }
    }
}