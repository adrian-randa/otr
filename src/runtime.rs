use std::cell::{Cell, RefCell};
use std::fmt::{Display, format};
use std::ops::Deref;
use std::rc::Weak;
use std::vec::IntoIter;
use std::{collections::HashMap, rc::Rc};

use derive_more::{Deref, IntoIterator};
use num::traits::identities;

use crate::compiler::CompilerError;
use crate::compiler::expression_parser::ExpressionParser;
use crate::lexer::token::{LiteralToken, ParenthesisType, PunctuationToken, Token};
use crate::runtime::environment::Environment;
use crate::runtime::expressions::ProcedureCallExpression;
use crate::runtime::procedures::{CompiledProcedure, Procedure};
use crate::runtime::scope::ScopeAddressant;

pub mod environment;
pub mod expressions;
pub mod module;
pub mod procedures;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

pub trait Expression: std::fmt::Debug {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError>;
}

#[derive(Debug)]
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
            Self::Struct(arg0) => {
                Value::Struct(Rc::new(RefCell::new(
                    arg0.borrow().as_ref().map(|obj| {
                        obj.clone()
                    })
                )))
            },
            Self::StructRef(arg0) => Self::StructRef(arg0.clone()),
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
            (Self::StructRef(l0), Self::StructRef(r0)) => {
                l0.upgrade() == r0.upgrade()
            },
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl TryFrom<LiteralToken> for Value {
    type Error = CompilerError;

    fn try_from(value: LiteralToken) -> Result<Self, Self::Error> {
        match value {
            LiteralToken::Null => {
                Ok(Self::Null)
            }
            LiteralToken::Integer(num) => {
                Ok(Self::Integer(
                    num.parse().map_err(|_| CompilerError {
                        message: format!("Could not parse '{}' as a whole number!", num)
                    })?
                ))
            },
            LiteralToken::Decimal(num) => {
                Ok(Self::Float(
                    num.parse().map_err(|_| CompilerError {
                        message: format!("Could not parse '{}' as a decimal number!", num)
                    })?
                ))
            },
            LiteralToken::Boolean(b) => {
                match &b as &str {
                    "true" => Ok(Self::Bool(true)),
                    "false" => Ok(Self::Bool(false)),
                    _ => Err(CompilerError { message: format!("Could not parse {} as a boolean!", b) })
                }
            },
            LiteralToken::Char(c) => {
                Ok(Self::Char(c.chars().next().ok_or(CompilerError {
                    message: format!("Could not parse {} as a char!", c)
                })?))
            },
            LiteralToken::String(str) => {
                Ok(Self::String(str))
            },
        }
    }
}

impl Value {
    pub fn get_type_id(&self) -> String {
        match self {
            Value::Null => "Null".into(),
            Value::Integer(_) => "Integer".into(),
            Value::Float(_) => "Float".into(),
            Value::String(_) => "String".into(),
            Value::Char(_) => "Char".into(),
            Value::Bool(_) => "Bool".into(),
            Value::Array(_) => "Array".into(),
            Value::Struct(object) => object
                .borrow()
                .as_ref()
                .map(|obj| obj.get_struct_id().to_string())
                .unwrap_or("Moved".into()),
            Value::StructRef(weak) => weak
                .upgrade()
                .map(|obj| obj.borrow()
                    .as_ref()
                    .map(|obj| obj.get_struct_id().to_string())
                    .unwrap_or("Moved".into()))
                .unwrap_or("Dropped".into()),
        }
    }

    pub fn query(&self, address: impl IntoIterator<Item = ScopeAddressant>, contained_module_id: &String) -> Result<Value, RuntimeError> {
        let mut address = address.into_iter();
        if let Some(addressant) = address.next() {
            match self {
                Value::Null | Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Char(_) |
                Value::Bool(_)  => Err(RuntimeError {
                    message: format!("Value '{:?}' doesn't acceppt addressant '{:?}'", self, addressant)
                }),
                Value::Array(arr) => {
                    if let ScopeAddressant::Index(i) = addressant {
                        arr.get(i).ok_or(RuntimeError {
                            message: format!("Index out of bounds! Index {} on array of length {}!", i, arr.len())
                        })?.query(address, contained_module_id)
                    } else {
                        Err(RuntimeError {
                            message: format!("Arrays only accept indexing addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::Struct(ref_cell) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let reference = ref_cell.borrow();
                        let obj = reference.as_ref().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let members = obj.get_members();
                        
                        if obj.get_struct_id().get_module_id() == contained_module_id {
                            members.get_member(&ident)?.query(address, contained_module_id)
                        } else {
                            members.get_public_member(&ident)?.query(address, contained_module_id)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::StructRef(weak) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let rc = weak.upgrade().ok_or(RuntimeError {
                            message: format!("Use of dropped value!")
                        })?;

                        let reference = rc.borrow();
                        let obj = reference.as_ref().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let members = obj.get_members();
                        
                        if obj.get_struct_id().get_module_id() == contained_module_id {
                            members.get_member(&ident)?.query(address, contained_module_id)
                        } else {
                            members.get_public_member(&ident)?.query(address, contained_module_id)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
            }
        } else {
            match self {
                Value::Null | Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Char(_) |
                Value::Bool(_) | Value::Array(_) | Value::StructRef(_) => Ok(self.clone()),
                Value::Struct(ref_cell) => {
                    if ref_cell.borrow().is_none() {
                        return Err(RuntimeError {
                            message: "Use of moved value!".into()
                        });
                    }

                    // Move value
                    let value = ref_cell.replace(None);

                    Ok(Value::Struct(Rc::new(RefCell::new(value))))
                }
            }
        }
    }

    pub fn reference(&self, address: impl IntoIterator<Item = ScopeAddressant>, contained_module_id: &String) -> Result<Value, RuntimeError> {
        let mut address = address.into_iter();
        if let Some(addressant) = address.next() {
            match self {
                Value::Null | Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Char(_) |
                Value::Bool(_)  => Err(RuntimeError {
                    message: format!("Value '{:?}' doesn't acceppt addressant '{:?}'", self, addressant)
                }),
                Value::Array(arr) => {
                    if let ScopeAddressant::Index(i) = addressant {
                        arr.get(i).ok_or(RuntimeError {
                            message: format!("Index out of bounds! Index {} on array of length {}!", i, arr.len())
                        })?.query(address, contained_module_id)
                    } else {
                        Err(RuntimeError {
                            message: format!("Arrays only accept indexing addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::Struct(ref_cell) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let reference = ref_cell.borrow();
                        let obj = reference.as_ref().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let members = obj.get_members();
                        
                        if obj.get_struct_id().get_module_id() == contained_module_id {
                            members.get_member(&ident)?.query(address, contained_module_id)
                        } else {
                            members.get_public_member(&ident)?.query(address, contained_module_id)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::StructRef(weak) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let rc = weak.upgrade().ok_or(RuntimeError {
                            message: format!("Use of dropped value!")
                        })?;

                        let reference = rc.borrow();
                        let obj = reference.as_ref().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let members = obj.get_members();
                        
                        if obj.get_struct_id().get_module_id() == contained_module_id {
                            members.get_member(&ident)?.query(address, contained_module_id)
                        } else {
                            members.get_public_member(&ident)?.query(address, contained_module_id)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
            }
        } else {
            match self {
                Value::Null | Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Char(_) |
                Value::Bool(_) | Value::Array(_) | Value::StructRef(_) => Err(RuntimeError {
                    message: format!("Can only reference owned structs. Found {:?}!", self)
                }),
                Value::Struct(ref_cell) => {
                    if ref_cell.borrow().is_none() {
                        return Err(RuntimeError {
                            message: "Use of moved value!".into()
                        });
                    }

                    // Reference
                    let weak = Rc::downgrade(&ref_cell.clone());

                    Ok(Value::StructRef(weak))
                }
            }
        }
    }

    pub fn set(&mut self, address: impl IntoIterator<Item = ScopeAddressant>, contained_module_id: &String, value: Value) -> Result<(), RuntimeError> {
        let mut address = address.into_iter();
        if let Some(addressant) = address.next() {
            match self {
                Value::Null | 
                Value::Integer(_) |
                Value::Float(_) |
                Value::String(_) |
                Value::Char(_) |
                Value::Bool(_)  => Err(RuntimeError {
                    message: format!("Value '{:?}' doesn't acceppt addressant '{:?}'", self, addressant)
                }),
                Value::Array(arr) => {
                    if let ScopeAddressant::Index(i) = addressant {
                        let len = arr.len();
                        arr.get_mut(i).ok_or(RuntimeError {
                            message: format!("Index out of bounds! Index {} on array of length {}!", i, len)
                        })?.set(address, contained_module_id, value)
                    } else {
                        Err(RuntimeError {
                            message: format!("Arrays only accept indexing addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::Struct(ref_cell) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let mut reference = ref_cell.borrow_mut();
                        let obj = reference.as_mut().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let module_id = obj.get_struct_id().get_module_id().clone();

                        let members = obj.get_members_mut();
                        
                        if &module_id == contained_module_id {
                            members.get_member_mut(&ident)?.set(address, contained_module_id, value)
                        } else {
                            members.get_public_member_mut(&ident)?.set(address, contained_module_id, value)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::StructRef(weak) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let rc = weak.upgrade().ok_or(RuntimeError {
                            message: format!("Use of dropped value!")
                        })?;

                        let mut reference = rc.borrow_mut();
                        let obj = reference.as_mut().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let module_id = obj.get_struct_id().get_module_id().clone();

                        let members = obj.get_members_mut();
                        
                        if &module_id == contained_module_id {
                            members.get_member_mut(&ident)?.set(address, contained_module_id, value)
                        } else {
                            members.get_public_member_mut(&ident)?.set(address, contained_module_id, value)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
            }
        } else {
            *self = value;
            Ok(())
        }
    }
    
    fn clone_variable(&self, address: IntoIter<ScopeAddressant>, contained_module_id: &String) -> Result<Value, RuntimeError> {
        let mut address = address.into_iter();
        if let Some(addressant) = address.next() {
            match self {
                Value::Null | Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Char(_) |
                Value::Bool(_)  => Err(RuntimeError {
                    message: format!("Value '{:?}' doesn't acceppt addressant '{:?}'", self, addressant)
                }),
                Value::Array(arr) => {
                    if let ScopeAddressant::Index(i) = addressant {
                        arr.get(i).ok_or(RuntimeError {
                            message: format!("Index out of bounds! Index {} on array of length {}!", i, arr.len())
                        })?.query(address, contained_module_id)
                    } else {
                        Err(RuntimeError {
                            message: format!("Arrays only accept indexing addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::Struct(ref_cell) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let reference = ref_cell.borrow();
                        let obj = reference.as_ref().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let members = obj.get_members();
                        
                        if obj.get_struct_id().get_module_id() == contained_module_id {
                            members.get_member(&ident)?.query(address, contained_module_id)
                        } else {
                            members.get_public_member(&ident)?.query(address, contained_module_id)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
                Value::StructRef(weak) => {
                    if let ScopeAddressant::Identifier(ident) = addressant {
                        let rc = weak.upgrade().ok_or(RuntimeError {
                            message: format!("Use of dropped value!")
                        })?;

                        let reference = rc.borrow();
                        let obj = reference.as_ref().ok_or(RuntimeError {
                            message: format!("Use of moved value!")
                        })?;

                        let members = obj.get_members();
                        
                        if obj.get_struct_id().get_module_id() == contained_module_id {
                            members.get_member(&ident)?.query(address, contained_module_id)
                        } else {
                            members.get_public_member(&ident)?.query(address, contained_module_id)
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Structs only accept identifier addressants. Found {:?}!", addressant)
                        })
                    }
                },
            }
        } else {
            if let Value::StructRef(weak) = self {
                let rc = weak.upgrade().ok_or(RuntimeError {
                    message: "Clone of dropped value".into()
                })?;

                Ok(Value::Struct(rc).clone())
            } else {
                Ok(self.clone())
            }
        }
    }
}

impl Expression for Value {
    fn eval(&self, _environment: &Environment) -> Result<Value, RuntimeError> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Member {
    is_public: bool,
    value: Value,
}

impl From<(bool, Value)> for Member {
    fn from((is_public, value): (bool, Value)) -> Self {
        Self { is_public, value }
    }
}

impl Member {
    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn get_value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn get_value_if_public(&self) -> Result<&Value, RuntimeError> {
        if self.is_public {
            Ok(&self.value)
        } else {
            Err(RuntimeError {
                message: "Tried to access a private field!".into(),
            })
        }
    }

    pub fn get_value_mut_if_public(&mut self) -> Result<&mut Value, RuntimeError> {
        if self.is_public {
            Ok(&mut self.value)
        } else {
            Err(RuntimeError {
                message: "Tried to access a private field!".into(),
            })
        }
    }

    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }

    pub fn set_if_public(&mut self, value: Value) -> Result<(), RuntimeError> {
        if self.is_public {
            self.value = value;
            Ok(())
        } else {
            Err(RuntimeError {
                message: "Tried to access a private field!".into(),
            })
        }
    }
    
    fn set(&mut self, value: Value) -> Result<(), RuntimeError> {
        self.value = value;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberMap {
    members: HashMap<String, Member>,
}

impl MemberMap {
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
        }
    }

    pub fn insert_member(&mut self, ident: String, value: Value, is_public: bool) -> Result<(), RuntimeError> {
        if self.members.insert(ident.clone(), Member { value, is_public }).is_some() {
            return Err(RuntimeError {
                message: format!("Cannot insert key '{}' into struct as it is already present!", ident)
            })
        }

        Ok(())
    }

    pub fn get_member(&self, ident: &String) -> Result<&Value, RuntimeError> {
        let member = self.members.get(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        Ok(member.get_value())
    }

    pub fn get_member_mut(&mut self, ident: &String) -> Result<&mut Value, RuntimeError> {
        let member = self.members.get_mut(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        Ok(member.get_value_mut())
    }

    pub fn get_public_member(&self, ident: &String) -> Result<&Value, RuntimeError> {
        let member = self.members.get(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        member.get_value_if_public()
    }

    pub fn get_public_member_mut(&mut self, ident: &String) -> Result<&mut Value, RuntimeError> {
        let member = self.members.get_mut(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        member.get_value_mut_if_public()
    }

    pub fn set_public_member(&mut self, ident: &String, value: Value) -> Result<(), RuntimeError> {
        let member = self.members.get_mut(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        member.set_if_public(value)
    }

    pub fn set_member(&mut self, ident: &String, value: Value) -> Result<(), RuntimeError> {
        let member = self.members.get_mut(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        member.set(value)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleAddress {
    module_id: String,
    identifier: String,
}

impl From<(&str, &str)> for ModuleAddress {
    fn from(value: (&str, &str)) -> Self {
        Self {
            module_id: value.0.to_string(),
            identifier: value.1.to_string(),
        }
    }
}

impl Display for ModuleAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.module_id, self.identifier)
    }
}

impl ModuleAddress {
    pub fn new(module_id: String, identifier: String) -> Self {
        Self {
            module_id,
            identifier,
        }
    }

    pub fn get_module_id(&self) -> &String {
        &self.module_id
    }

    pub fn get_identifier(&self) -> &String {
        &self.identifier
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    //TODO: Remove public visibility
    pub struct_id: ModuleAddress,
    pub members: MemberMap,
}

impl Struct {
    pub fn new(struct_id: ModuleAddress) -> Self {
        Self {
            struct_id,
            members: MemberMap::new(),
        }
    }

    pub fn get_struct_id(&self) -> &ModuleAddress {
        &self.struct_id
    }

    pub fn get_members(&self) -> &MemberMap {
        &self.members
    }

    pub fn get_members_mut(&mut self) -> &mut MemberMap {
        &mut self.members
    }
}


#[derive(Debug)]
pub struct RuntimeObject {
    pub(crate) base_environement: Environment,
    pub(crate) entrypoint: Option<ModuleAddress>
}

impl RuntimeObject {
    pub(crate) fn new() -> Self {
        Self {
            base_environement: Environment::new("".into()),
            entrypoint: None,
        }
    }

    pub fn execute(self) -> Result<Value, RuntimeError> {
        let entrypoint = self.entrypoint.ok_or(RuntimeError {
            message: "No specified entrypoint!".into()
        })?;

        let main_expression = ProcedureCallExpression::new(
            entrypoint,
            Vec::new()
        );

        main_expression.eval(&self.base_environement)
    }
}

pub mod scope;