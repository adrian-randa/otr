#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Null,
    Integer,
    Float,
    String,
    Char,
    Bool,
    Array,
    Struct { struct_id: ModuleAddress },
    Moved,
    Dropped,
    Type,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let representation = match self {
            Type::Null => "Null",
            Type::Integer => "Integer",
            Type::Float => "Float",
            Type::String => "String",
            Type::Char => "Char",
            Type::Bool => "Bool",
            Type::Array => "Array",
            Type::Struct { struct_id } => &struct_id.to_string(),
            Type::Moved => "Moved",
            Type::Dropped => "Dropped",
            Type::Type => "Type",
        };

        write!(f, "{}", representation)
    }
}

macro_rules! id {
    ($value:ident: $id0:ident $(, $id:ident)+) => {
        match $value {
            PrimitiveTypeToken::$id0 => Self::$id0,
            $(
                PrimitiveTypeToken::$id => Self::$id,
            )+
        }
    };
}

impl From<PrimitiveTypeToken> for Type {
    fn from(value: PrimitiveTypeToken) -> Self {
        id!(value: Null, Integer, Float, Bool, Char, String, Array, Moved, Dropped, Type)
    }
}