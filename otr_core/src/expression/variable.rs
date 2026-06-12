use derive_more::{Deref, DerefMut, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::expression::Expression;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableExpression {
    address: VariableAddress,
    access_mode: VariableAccessMode,
}

impl VariableExpression {
    pub fn new(address: VariableAddress, access_mode: VariableAccessMode) -> Self {
        Self { address, access_mode }
    }

    pub fn get_address(&self) -> &VariableAddress {
        &self.address
    }

    pub fn get_access_mode(&self) -> VariableAccessMode {
        self.access_mode
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum VariableAccessMode {
    Move,
    Clone,
    Ref,
    TypeOf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableAddressant {
    StackIndex(usize),
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

#[derive(Debug, Clone, PartialEq, Deref, DerefMut, IntoIterator, Serialize, Deserialize)]
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