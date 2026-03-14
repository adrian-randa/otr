use std::collections::HashMap;
use itertools::Itertools;

use crate::{core::value::Value, error::{Result, runtime_error::RuntimeError}};

#[derive(Debug, Clone, PartialEq)]
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
    pub fn get_unchecked(&self) -> &Value {
        &self.value
    }

    pub fn get_unchecked_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn get(&self) -> Result<&Value> {
        if self.is_public {
            Ok(&self.value)
        } else {
            Err(RuntimeError::FieldIsPrivate.boxed())
        }
    }

    pub fn get_mut(&mut self) -> Result<&mut Value> {
        if self.is_public {
            Ok(self.get_unchecked_mut())
        } else {
            Err(RuntimeError::FieldIsPrivate.boxed())
        }
    }

    fn set_unchecked(&mut self, value: Value) {
        self.value = value;
    }

    pub fn set(&mut self, value: Value) -> Result<()> {
        if self.is_public {
            Ok(self.set_unchecked(value))
        } else {
            Err(RuntimeError::FieldIsPrivate.boxed())
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct MemberMap {
    members: HashMap<String, Member>,
}

impl std::fmt::Display for MemberMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", self.members.iter()
            .map(|(label, value)| { label.to_string() + ": " + &value.get_unchecked().to_string() })
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

    pub fn insert(&mut self, ident: String, value: Value, is_public: bool) -> Result<()> {
        if self
            .members
            .insert(ident.clone(), Member { value, is_public })
            .is_some()
        {
            return Err(RuntimeError::KeyAlreadyPresent { key: ident }.boxed());
        }

        Ok(())
    }

    pub fn get_unchecked(&self, ident: &String) -> Result<&Value> {
        let member = self.members.get(ident).ok_or(
            RuntimeError::NoSuchMember {
                member_identifier: ident.clone(),
            }
            .boxed(),
        )?;

        Ok(member.get_unchecked())
    }

    pub fn get_unchecked_mut(&mut self, ident: &String) -> Result<&mut Value> {
        let member = self.members.get_mut(ident).ok_or(
            RuntimeError::NoSuchMember {
                member_identifier: ident.clone(),
            }
            .boxed(),
        )?;

        Ok(member.get_unchecked_mut())
    }

    pub fn get(&self, ident: &String) -> Result<&Value> {
        let member = self.members.get(ident).ok_or(
            RuntimeError::NoSuchMember {
                member_identifier: ident.clone(),
            }
            .boxed(),
        )?;

        member.get()
    }

    pub fn get_mut(&mut self, ident: &String) -> Result<&mut Value> {
        let member = self.members.get_mut(ident).ok_or(
            RuntimeError::NoSuchMember {
                member_identifier: ident.clone(),
            }
            .boxed(),
        )?;

        member.get_mut()
    }

    pub fn set(&mut self, ident: &String, value: Value) -> Result<()> {
        let member = self.members.get_mut(ident).ok_or(
            RuntimeError::NoSuchMember {
                member_identifier: ident.clone(),
            }
            .boxed(),
        )?;

        member.set(value)
    }

    pub fn set_unchecked(&mut self, ident: &String, value: Value) -> Result<()> {
        let member = self.members.get_mut(ident).ok_or(
            RuntimeError::NoSuchMember {
                member_identifier: ident.clone(),
            }
            .boxed(),
        )?;

        member.set_unchecked(value);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}