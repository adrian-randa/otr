use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
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

    pub(crate) fn with_member(mut self, ident: String, value: Value, is_public: bool) -> Result<Self> {
        self.get_members_mut().insert(ident, value, is_public)?;
        Ok(self)
    }
}

impl std::fmt::Display for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{{}}}",
            self.get_struct_id().to_string(),
            self.get_members()
                .members
                .iter()
                .map(|(label, value)| { label.to_string() + ": " + &value.get_unchecked().to_string() })
                .join(", ")
        )
    }
}