use std::{cell::RefCell, rc::{Rc, Weak}, vec::IntoIter};

use itertools::Itertools;

use crate::{core::{expression::variable::VariableAddressant, r#struct::Struct, r#type::Type}, error::{Result, context::VariableContextDecorator, runtime_error::RuntimeError}};

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

    pub(crate) fn get(
        &self,
        address: IntoIter<VariableAddressant>,
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
        address: IntoIter<VariableAddressant>,
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
        address: IntoIter<VariableAddressant>,
        contained_module_id: &String,
    ) -> Result<Value> {
        self.apply_to_submember(Self::get_value_type, address, contained_module_id, ())
    }

    pub(crate) fn get_value_type(&self, _: ()) -> Result<Value> {
        Ok(Value::Type(self.get_type_id()))
    }

    pub(crate) fn set(
        &mut self,
        address: IntoIter<VariableAddressant>,
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

    pub(crate) fn clone_member(&self, address: IntoIter<VariableAddressant>, contained_module_id: &String) -> Result<Value> {
        self.apply_to_submember(Self::clone_value, address, contained_module_id, ())
    }

    pub(crate) fn apply_to_submember<Args, T>(
        &self,
        function: impl Fn(&Self, Args) -> Result<T>,
        mut address: IntoIter<VariableAddressant>,
        contained_module_id: &String,
        args: Args
    ) -> Result<T> {
        if let Some(addressant) = address.next() {
            let member_ident = format!("{:?}", addressant);
            let result = {
                match self {
                    Value::Array(arr) => {
                        if let VariableAddressant::Index(i) = addressant {
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
                        if let VariableAddressant::Identifier(ident) = addressant {
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
                        if let VariableAddressant::Identifier(ident) = addressant {
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
        mut address: IntoIter<VariableAddressant>,
        contained_module_id: &String,
        args: Args
    ) -> Result<T> {
        if let Some(addressant) = address.next() {
            let member_ident = format!("{:?}", addressant);
            let result = {
                match self {
                    Value::Array(arr) => {
                        if let VariableAddressant::Index(i) = addressant {
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
                        if let VariableAddressant::Identifier(ident) = addressant {
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
                        if let VariableAddressant::Identifier(ident) = addressant {
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
