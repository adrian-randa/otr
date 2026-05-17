use std::cell::RefCell;
use std::rc::Rc;
use std::vec::IntoIter;

use otr_core::member::MemberMap;
use otr_core::{expression::variable::VariableAddressant, value::Value};
use otr_core::Result;
use crate::error::context::VariableContextDecorator;
use crate::error::RuntimeError;

pub(crate) fn compare(lhs: &Value, rhs: &Value) -> Result<std::cmp::Ordering> {
    use Value::*;
    match (lhs, rhs) {
        (Integer(l), Integer(r)) => Ok(l.cmp(r)),
        (Float(l), Float(r)) => Ok(l.partial_cmp(r).ok_or(
            RuntimeError::Unknown { message: format!("Could not compare {l} to {r}") }.boxed()
        ))?,
        (String(l), String(r)) => Ok(l.cmp(r)),
        (Char(l), Char(r)) => Ok(l.cmp(r)),

        (l, r) => Err(RuntimeError::Unknown {
            message: format!(
                "Ordering is undefined on {} and {}!",
                l.get_type_id(),
                r.get_type_id()
            ),
        }
        .boxed()),
    }
}

fn try_get_value<'a>(member_map: &'a MemberMap, member_ident: &'a str) -> Result<&'a Value> {
    member_map
        .get_value(member_ident)
        .ok_or(RuntimeError::NoSuchMember { member_identifier: member_ident.to_string() }.boxed())
}

fn try_get_value_if_public<'a>(member_map: &'a MemberMap, member_ident: &'a str) -> Result<&'a Value> {
    let is_public = member_map
        .is_public(member_ident)
        .ok_or(RuntimeError::NoSuchMember { member_identifier: member_ident.to_string() }.boxed())?;

    if is_public {
        member_map
            .get_value(member_ident)
            .ok_or(RuntimeError::NoSuchMember { member_identifier: member_ident.to_string() }.boxed())
    } else {
        Err(RuntimeError::FieldIsPrivate.boxed())
    }
}

fn try_get_value_mut<'a>(member_map: &'a mut MemberMap, member_ident: &'a str) -> Result<&'a mut Value> {
    member_map
        .get_value_mut(member_ident)
        .ok_or(RuntimeError::NoSuchMember { member_identifier: member_ident.to_string() }.boxed())
}

fn try_get_value_mut_if_public<'a>(member_map: &'a mut MemberMap, member_ident: &'a str) -> Result<&'a mut Value> {
    let is_public = member_map
        .is_public(member_ident)
        .ok_or(RuntimeError::NoSuchMember { member_identifier: member_ident.to_string() }.boxed())?;

    if is_public {
        member_map
            .get_value_mut(member_ident)
            .ok_or(RuntimeError::NoSuchMember { member_identifier: member_ident.to_string() }.boxed())
    } else {
        Err(RuntimeError::FieldIsPrivate.boxed())
    }
}

pub(crate) fn get(
    value: &Value,
    address: IntoIter<VariableAddressant>,
    contained_module_id: &String,
) -> Result<Value> {
    apply_to_submember(get_value, value, address, contained_module_id, ())
}

pub(crate) fn get_value(value: &Value, _: ()) -> Result<Value> {
    match value {
        Value::Struct(ref_cell) => {
            if ref_cell.borrow().is_none() {
                return Err(RuntimeError::UseOfMovedValue.boxed());
            }

            // Move value
            let value = ref_cell.replace(None);

            Ok(Value::Struct(Rc::new(RefCell::new(value))))
        }
        _ => Ok(value.clone()),
    }
}

pub fn reference(
    value: &Value,
    address: IntoIter<VariableAddressant>,
    contained_module_id: &String,
) -> Result<Value> {
    apply_to_submember(reference_value, value, address, contained_module_id, ())
}

pub(crate) fn reference_value(value: &Value, _: ()) -> Result<Value> {
    match value {
        Value::Struct(ref_cell) => {
            if ref_cell.borrow().is_none() {
                return Err(RuntimeError::UseOfMovedValue.boxed());
            }

            // Reference
            let weak = Rc::downgrade(&ref_cell.clone());

            Ok(Value::StructRef(weak))
        }
        _ => Err(RuntimeError::CannotReference {
            ty: value.get_type_id(),
        }
        .boxed()),
    }
}

pub(crate) fn get_type(
    value: &Value,
    address: IntoIter<VariableAddressant>,
    contained_module_id: &String,
) -> Result<Value> {
    apply_to_submember(get_value_type, value, address, contained_module_id, ())
}

pub(crate) fn get_value_type(value: &Value, _: ()) -> Result<Value> {
    Ok(Value::Type(value.get_type_id()))
}

pub(crate) fn set(
    value: &mut Value,
    address: IntoIter<VariableAddressant>,
    contained_module_id: &String,
    new_value: Value,
) -> Result<()> {
    apply_to_submember_mut(set_value, value, address, contained_module_id, new_value)
}

pub(crate) fn set_value(value: &mut Value, new_value: Value) -> Result<()> {
    *value = new_value;
    Ok(())
}

pub(crate) fn clone_value(value: &Value, _: ()) -> Result<Value> {
    if let Value::StructRef(weak) = value {
        let rc = weak
            .upgrade()
            .ok_or(RuntimeError::UseOfDroppedValue.boxed())?;

        Ok(Value::Struct(rc).clone())
    } else {
        Ok(value.clone())
    }
}

pub(crate) fn clone_member(value: &Value, address: IntoIter<VariableAddressant>, contained_module_id: &String) -> Result<Value> {
    apply_to_submember(clone_value, value, address, contained_module_id, ())
}

pub(crate) fn apply_to_submember<Args, T>(
    function: impl Fn(&Value, Args) -> Result<T>,
    value: &Value,
    mut address: IntoIter<VariableAddressant>,
    contained_module_id: &String,
    args: Args
) -> Result<T> {
    if let Some(addressant) = address.next() {
        let member_ident = format!("{:?}", addressant);
        let result = {
            match value {
                Value::Array(arr) => {
                    if let VariableAddressant::Index(i) = addressant {
                        let arr_len = arr.len();
                        let value = arr.get(i)
                            .ok_or(
                                RuntimeError::IndexOutOfBounds {
                                    array_length: arr_len,
                                    index: i,
                                }
                                .boxed(),
                            )?;
                        apply_to_submember(function, value, address, contained_module_id, args)
                    } else {
                        Err(RuntimeError::MembersNotAccepted {
                            ty: value.get_type_id(),
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
                            let value = try_get_value(members, &ident)?;
                            
                            apply_to_submember(function, value, address, contained_module_id, args)
                        } else {
                            let value = try_get_value_if_public(members, &ident)?;
                            
                            apply_to_submember(function, value, address, contained_module_id, args)
                        }
                    } else {
                        Err(RuntimeError::IndexingNotAccepted {
                            ty: value.get_type_id(),
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
                            let value = try_get_value(members, &ident)?;
                            
                            apply_to_submember(function, value, address, contained_module_id, args)
                        } else {
                            let value = try_get_value_if_public(members, &ident)?;
                            
                            apply_to_submember(function, value, address, contained_module_id, args)
                        }
                    } else {
                        Err(RuntimeError::IndexingNotAccepted {
                            ty: value.get_type_id(),
                        }
                        .boxed())
                    }
                }
                _ => Err(RuntimeError::AddressantsNotAccepted {
                    ty: value.get_type_id(),
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
        function(value, args)
    }
}

pub(crate) fn apply_to_submember_mut<Args, T>(
    function: impl Fn(&mut Value, Args) -> Result<T>,
    value: &mut Value,
    mut address: IntoIter<VariableAddressant>,
    contained_module_id: &String,
    args: Args
) -> Result<T> {
    if let Some(addressant) = address.next() {
        let member_ident = format!("{:?}", addressant);
        let result = {
            match value {
                Value::Array(arr) => {
                    if let VariableAddressant::Index(i) = addressant {
                        let arr_len = arr.len();
                        let value = arr.get_mut(i)
                            .ok_or(
                                RuntimeError::IndexOutOfBounds {
                                    array_length: arr_len,
                                    index: i,
                                }
                                .boxed(),
                            )?;
                        apply_to_submember_mut(function, value, address, contained_module_id, args)
                    } else {
                        Err(RuntimeError::MembersNotAccepted {
                            ty: value.get_type_id(),
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
                            let value = try_get_value_mut(members, &ident)?;
                            
                            apply_to_submember_mut(function, value, address, contained_module_id, args)
                        } else {
                            let value = try_get_value_mut_if_public(members, &ident)?;
                            
                            apply_to_submember_mut(function, value, address, contained_module_id, args)
                        }
                    } else {
                        Err(RuntimeError::IndexingNotAccepted {
                            ty: value.get_type_id(),
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
                            let value = try_get_value_mut(members, &ident)?;
                            
                            apply_to_submember_mut(function, value, address, contained_module_id, args)
                        } else {
                            let value = try_get_value_mut_if_public(members, &ident)?;
                            
                            apply_to_submember_mut(function, value, address, contained_module_id, args)
                        }
                    } else {
                        Err(RuntimeError::IndexingNotAccepted {
                            ty: value.get_type_id(),
                        }
                        .boxed())
                    }
                }
                _ => Err(RuntimeError::AddressantsNotAccepted {
                    ty: value.get_type_id(),
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
        function(value, args)
    }
}