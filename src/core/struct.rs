use serde::{Deserialize, Serialize};

use crate::{core::{member::MemberMap, module::ModuleAddress, value::Value}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Struct {
    struct_id: ModuleAddress,
    members: MemberMap,
}

impl Struct {
    pub(crate) fn new(struct_id: ModuleAddress) -> Self {
        Self {
            struct_id,
            members: MemberMap::new(),
        }
    }

    pub(crate) fn get_struct_id(&self) -> &ModuleAddress {
        &self.struct_id
    }

    pub(crate) fn get_members(&self) -> &MemberMap {
        &self.members
    }

    pub(crate) fn get_members_mut(&mut self) -> &mut MemberMap {
        &mut self.members
    }

    pub(crate) fn with_member(mut self, ident: String, value: Value, is_public: bool) -> Self {
        self.get_members_mut().insert(ident, value, is_public);
        self
    }
}

impl std::fmt::Display for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.get_struct_id().to_string(),
            self.get_members()
        )
    }
}