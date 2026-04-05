use std::{cell::RefCell, rc::{Rc, Weak}};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::core::{r#struct::Struct, r#type::Type};

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Array(Vec<Value>),
    Struct(Rc<RefCell<Option<Struct>>>),
    StructRef(Weak<RefCell<Option<Struct>>>),
    Type(Type),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Integer(arg0) => Self::Integer(arg0.clone()),
            Self::Float(arg0) => Self::Float(arg0.clone()),
            Self::String(arg0) => Self::String(arg0.clone()),
            Self::Char(arg0) => Self::Char(arg0.clone()),
            Self::Bool(arg0) => Self::Bool(arg0.clone()),
            Self::Array(arg0) => Self::Array(arg0.clone()),
            Self::Struct(arg0) => Value::Struct(Rc::new(RefCell::new(
                arg0.borrow().as_ref().map(|obj| obj.clone()),
            ))),
            Self::StructRef(arg0) => Self::StructRef(arg0.clone()),
            Self::Type(arg0) => Self::Type(arg0.clone()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Char(l0), Self::Char(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Struct(l0), Self::Struct(r0)) => l0 == r0,
            (Self::StructRef(l0), Self::StructRef(r0)) => l0.upgrade() == r0.upgrade(),
            (Self::Type(l0), Self::Type(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
impl Value {
    pub(crate) fn get_type_id(&self) -> Type {
        match self {
            Value::Null => Type::Null,
            Value::Integer(_) => Type::Integer,
            Value::Float(_) => Type::Float,
            Value::String(_) => Type::String,
            Value::Char(_) => Type::Char,
            Value::Bool(_) => Type::Bool,
            Value::Array(_) => Type::Array,
            Value::Struct(object) => object
                .borrow()
                .as_ref()
                .map(|obj| Type::Struct {
                    struct_id: obj.get_struct_id().clone(),
                })
                .unwrap_or(Type::Moved),
            Value::StructRef(weak) => weak
                .upgrade()
                .map(|obj| {
                    obj.borrow()
                        .as_ref()
                        .map(|obj| Type::Struct {
                            struct_id: obj.get_struct_id().clone(),
                        })
                        .unwrap_or(Type::Moved)
                })
                .unwrap_or(Type::Dropped),
            Value::Type(_) => Type::Type,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "Null"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(values) => write!(
                f,
                "[{}]",
                values.into_iter().map(|item| item.to_string()).join(", ")
            ),
            Value::Struct(ref_cell) => write!(
                f,
                "{}",
                ref_cell
                    .borrow()
                    .as_ref()
                    .map(|obj| obj.to_string())
                    .unwrap_or("Moved".to_string())
            ),
            Value::StructRef(weak) => write!(
                f,
                "{}",
                weak.upgrade()
                    .map(|rc| {
                        rc.borrow()
                            .as_ref()
                            .map(|obj| obj.to_string())
                            .unwrap_or("Moved".to_string())
                    })
                    .unwrap_or("Dropped".to_string())
            ),
            Value::Type(t) => write!(f, "{}", t),
        }
    }
}
