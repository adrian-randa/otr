use std::collections::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::core::value::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Member {
    is_public: bool,
    value: Value,
}

impl From<(bool, Value)> for Member {
    fn from((is_public, value): (bool, Value)) -> Self {
        Self { is_public, value }
    }
}

impl Member {
    pub fn get(&self) -> &Value {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    fn set(&mut self, value: Value) {
        self.value = value;
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberMap {
    members: HashMap<String, Member>,
}

impl std::fmt::Display for MemberMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", self.members.iter()
            .map(|(label, value)| { label.to_string() + ": " + &value.get().to_string() })
            .join(", ")
        )
    }
}

impl MemberMap {
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
        }
    }

    pub fn insert(&mut self, ident: String, value: Value, is_public: bool) -> Option<Member> {
        self.members.insert(ident.clone(), Member { value, is_public })
    }

    pub fn get_value(&self, ident: &str) -> Option<&Value> {
        Some(self.members.get(ident)?.get())
    }

    pub fn get_value_mut(&mut self, ident: &str) -> Option<&mut Value> {
        Some(self.members.get_mut(ident)?.get_mut())
    }

    pub fn set(&mut self, ident: &str, value: Value) -> Option<()> {
        let member = self.members.get_mut(ident)?;

        member.set(value);
        Some(())
    }

    pub fn is_public(&self, ident: &str) -> Option<bool> {
        self.members.get(ident).map(|member| member.is_public)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}