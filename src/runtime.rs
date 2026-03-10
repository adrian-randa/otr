use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Weak;
use std::vec::IntoIter;
use std::{collections::HashMap, rc::Rc};

use itertools::Itertools;

use crate::error::compiler_error::CompilerError;
use crate::error::context::{HintContextDecorator, VariableContextDecorator};
use crate::error::runtime_error::RuntimeError;
use crate::error::Error;
use crate::lexer::token::{
    LiteralToken, PrimitiveTypeToken,
};
use crate::runtime::environment::Environment;
use crate::runtime::expressions::ProcedureCallExpression;
use crate::runtime::scope::ScopeAddressant;

use crate::error::Result;

pub mod environment;
pub mod expressions;
pub mod module;
pub mod procedures;

pub trait Expression: std::fmt::Debug {
    fn eval(&self, environment: &Environment) -> Result<Value>;
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
    Type(Type),
}

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

impl TryFrom<LiteralToken> for Value {
    type Error = Box<dyn Error>;

    fn try_from(value: LiteralToken) -> std::result::Result<Self, Self::Error> {
        match value {
            LiteralToken::Null => Ok(Self::Null),
            LiteralToken::Integer(num) => Ok(Self::Integer(num.parse().map_err(|_| {
                CompilerError::LiteralParseError {
                    ty: Type::Integer,
                    literal: num,
                }
                .boxed()
            })?)),
            LiteralToken::Float(num) => Ok(Self::Float(num.parse().map_err(|_| {
                CompilerError::LiteralParseError {
                    ty: Type::Float,
                    literal: num,
                }
                .boxed()
            })?)),
            LiteralToken::Boolean(b) => match &b as &str {
                "true" => Ok(Self::Bool(true)),
                "false" => Ok(Self::Bool(false)),
                _ => Err(CompilerError::LiteralParseError {
                    ty: Type::Bool,
                    literal: b,
                }
                .boxed()),
            },
            LiteralToken::Char(c) => Ok(Self::Char(
                c.chars().next().ok_or(
                    CompilerError::LiteralParseError {
                        ty: Type::Char,
                        literal: c,
                    }
                    .boxed(),
                )?,
            )),
            LiteralToken::String(str) => Ok(Self::String(str)),
            LiteralToken::Type(ty) => Ok(Self::Type(Type::from(ty))),
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

    pub(crate) fn get(
        &self,
        address: IntoIter<ScopeAddressant>,
        contained_module_id: &String,
    ) -> Result<Value> {
        self.apply_to_submember(Self::get_value, address, contained_module_id, ())
    }

    pub(crate) fn get_value(&self, _: ()) -> Result<Value> {
        match self {
            Value::Struct(ref_cell) => {
                if ref_cell.borrow().is_none() {
                    return Err(RuntimeError::UseOfMovedValue.boxed());
                }

                // Move value
                let value = ref_cell.replace(None);

                Ok(Value::Struct(Rc::new(RefCell::new(value))))
            }
            _ => Ok(self.clone()),
        }
    }

    pub fn reference(
        &self,
        address: IntoIter<ScopeAddressant>,
        contained_module_id: &String,
    ) -> Result<Value> {
        self.apply_to_submember(Self::reference_value, address, contained_module_id, ())
    }

    pub(crate) fn reference_value(&self, _: ()) -> Result<Value> {
        match self {
            Value::Struct(ref_cell) => {
                if ref_cell.borrow().is_none() {
                    return Err(RuntimeError::UseOfMovedValue.boxed());
                }

                // Reference
                let weak = Rc::downgrade(&ref_cell.clone());

                Ok(Value::StructRef(weak))
            }
            _ => Err(RuntimeError::CannotReference {
                ty: self.get_type_id(),
            }
            .boxed()),
        }
    }

    pub(crate) fn get_type(
        &self,
        address: IntoIter<ScopeAddressant>,
        contained_module_id: &String,
    ) -> Result<Value> {
        self.apply_to_submember(Self::get_value_type, address, contained_module_id, ())
    }

    pub(crate) fn get_value_type(&self, _: ()) -> Result<Value> {
        Ok(Value::Type(self.get_type_id()))
    }

    pub(crate) fn set(
        &mut self,
        address: IntoIter<ScopeAddressant>,
        contained_module_id: &String,
        value: Value,
    ) -> Result<()> {
        self.apply_to_submember_mut(Self::set_value, address, contained_module_id, value)
    }

    pub(crate) fn set_value(&mut self, value: Value) -> Result<()> {
        *self = value;
        Ok(())
    }

    pub(crate) fn clone_value(&self, _: ()) -> Result<Value> {
        if let Value::StructRef(weak) = self {
            let rc = weak
                .upgrade()
                .ok_or(RuntimeError::UseOfDroppedValue.boxed())?;

            Ok(Value::Struct(rc).clone())
        } else {
            Ok(self.clone())
        }
    }

    pub(crate) fn clone_member(&self, address: IntoIter<ScopeAddressant>, contained_module_id: &String) -> Result<Value> {
        self.apply_to_submember(Self::clone_value, address, contained_module_id, ())
    }

    pub(crate) fn apply_to_submember<Args, T>(
        &self,
        function: impl Fn(&Self, Args) -> Result<T>,
        mut address: IntoIter<ScopeAddressant>,
        contained_module_id: &String,
        args: Args
    ) -> Result<T> {
        if let Some(addressant) = address.next() {
            let member_ident = format!("{:?}", addressant);
            let result = {
                match self {
                    Value::Array(arr) => {
                        if let ScopeAddressant::Index(i) = addressant {
                            let arr_len = arr.len();
                            arr.get(i)
                                .ok_or(
                                    RuntimeError::IndexOutOfBounds {
                                        array_length: arr_len,
                                        index: i,
                                    }
                                    .boxed(),
                                )?
                                .apply_to_submember(function, address, contained_module_id, args)
                        } else {
                            Err(RuntimeError::MembersNotAccepted {
                                ty: self.get_type_id(),
                            }
                            .boxed())
                        }
                    }
                    Value::Struct(ref_cell) => {
                        if let ScopeAddressant::Identifier(ident) = addressant {
                            let reference = ref_cell.borrow();
                            let obj = reference
                                .as_ref()
                                .ok_or(RuntimeError::UseOfMovedValue.boxed())?;

                            let module_id_matches = obj.get_struct_id().get_module_id()  == contained_module_id;

                            let members = obj.get_members();

                            if module_id_matches {
                                members
                                    .get_unchecked(&ident)?
                                    .apply_to_submember(function, address, contained_module_id, args)
                            } else {
                                members
                                    .get(&ident)?
                                    .apply_to_submember(function, address, contained_module_id, args)
                            }
                        } else {
                            Err(RuntimeError::IndexingNotAccepted {
                                ty: self.get_type_id(),
                            }
                            .boxed())
                        }
                    }
                    Value::StructRef(weak) => {
                        if let ScopeAddressant::Identifier(ident) = addressant {
                            let rc = weak
                                .upgrade()
                                .ok_or(RuntimeError::UseOfDroppedValue.boxed())?;

                            let reference = rc.borrow();
                            let obj = reference
                                .as_ref()
                                .ok_or(RuntimeError::UseOfMovedValue.boxed())?;

                            let module_id_matches = obj.get_struct_id().get_module_id()  == contained_module_id;

                            let members = obj.get_members();

                            if module_id_matches {
                                members
                                    .get_unchecked(&ident)?
                                    .apply_to_submember(function, address, contained_module_id, args)
                            } else {
                                members
                                    .get(&ident)?
                                    .apply_to_submember(function, address, contained_module_id, args)
                            }
                        } else {
                            Err(RuntimeError::IndexingNotAccepted {
                                ty: self.get_type_id(),
                            }
                            .boxed())
                        }
                    }
                    _ => Err(RuntimeError::AddressantsNotAccepted {
                        ty: self.get_type_id(),
                    }
                    .boxed()),
                }
            };

            result.map_err(|error| {
                {
                    VariableContextDecorator {
                        error,
                        member_ident,
                    }
                }
                .boxed()
            })
        } else {
            function(self, args)
        }
    }

    pub(crate) fn apply_to_submember_mut<Args, T>(
        &mut self,
        function: impl Fn(&mut Self, Args) -> Result<T>,
        mut address: IntoIter<ScopeAddressant>,
        contained_module_id: &String,
        args: Args
    ) -> Result<T> {
        if let Some(addressant) = address.next() {
            let member_ident = format!("{:?}", addressant);
            let result = {
                match self {
                    Value::Array(arr) => {
                        if let ScopeAddressant::Index(i) = addressant {
                            let arr_len = arr.len();
                            arr.get_mut(i)
                                .ok_or(
                                    RuntimeError::IndexOutOfBounds {
                                        array_length: arr_len,
                                        index: i,
                                    }
                                    .boxed(),
                                )?
                                .apply_to_submember_mut(function, address, contained_module_id, args)
                        } else {
                            Err(RuntimeError::MembersNotAccepted {
                                ty: self.get_type_id(),
                            }
                            .boxed())
                        }
                    }
                    Value::Struct(ref_cell) => {
                        if let ScopeAddressant::Identifier(ident) = addressant {
                            let mut reference = ref_cell.borrow_mut();
                            let obj = reference
                                .as_mut()
                                .ok_or(RuntimeError::UseOfMovedValue.boxed())?;

                            let module_id_matches = obj.get_struct_id().get_module_id()  == contained_module_id;

                            let members = obj.get_members_mut();

                            if module_id_matches {
                                members
                                    .get_unchecked_mut(&ident)?
                                    .apply_to_submember_mut(function, address, contained_module_id, args)
                            } else {
                                members
                                    .get_mut(&ident)?
                                    .apply_to_submember_mut(function, address, contained_module_id, args)
                            }
                        } else {
                            Err(RuntimeError::IndexingNotAccepted {
                                ty: self.get_type_id(),
                            }
                            .boxed())
                        }
                    }
                    Value::StructRef(weak) => {
                        if let ScopeAddressant::Identifier(ident) = addressant {
                            let rc = weak
                                .upgrade()
                                .ok_or(RuntimeError::UseOfDroppedValue.boxed())?;

                            let mut reference = rc.borrow_mut();
                            let obj = reference
                                .as_mut()
                                .ok_or(RuntimeError::UseOfMovedValue.boxed())?;

                            let module_id_matches = obj.get_struct_id().get_module_id()  == contained_module_id;

                            let members = obj.get_members_mut();

                            if module_id_matches {
                                members
                                    .get_unchecked_mut(&ident)?
                                    .apply_to_submember_mut(function, address, contained_module_id, args)
                            } else {
                                members
                                    .get_mut(&ident)?
                                    .apply_to_submember_mut(function, address, contained_module_id, args)
                            }
                        } else {
                            Err(RuntimeError::IndexingNotAccepted {
                                ty: self.get_type_id(),
                            }
                            .boxed())
                        }
                    }
                    _ => Err(RuntimeError::AddressantsNotAccepted {
                        ty: self.get_type_id(),
                    }
                    .boxed()),
                }
            };

            result.map_err(|error| {
                {
                    VariableContextDecorator {
                        error,
                        member_ident,
                    }
                }
                .boxed()
            })
        } else {
            function(self, args)
        }
    }
}

impl Expression for Value {
    fn eval(&self, _environment: &Environment) -> Result<Value> {
        Ok(self.clone())
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
    pub(crate) fn new(module_id: String, identifier: String) -> Self {
        Self {
            module_id,
            identifier,
        }
    }

    pub(crate) fn get_module_id(&self) -> &String {
        &self.module_id
    }

    pub(crate) fn get_identifier(&self) -> &String {
        &self.identifier
    }
}

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

#[derive(Debug)]
pub struct RuntimeObject {
    pub(crate) base_environement: Environment,
    pub(crate) entrypoint: Option<ModuleAddress>,
}

impl RuntimeObject {
    pub(crate) fn new() -> Self {
        Self {
            base_environement: Environment::new("".into()),
            entrypoint: None,
        }
    }

    pub fn execute(self) -> Result<Value> {
        let entrypoint = self.entrypoint.ok_or(
            HintContextDecorator {
                error: RuntimeError::NoEntrypoint.boxed(),
                message: "If you want to run the specified file as a script, please annotate a procedure as the entrypoint.".into()
            }.boxed()
        )?;

        let main_expression = ProcedureCallExpression::new(entrypoint, Vec::new());

        main_expression.eval(&self.base_environement)
    }
}

pub mod scope;
