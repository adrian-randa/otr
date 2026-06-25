use std::{cell::RefCell, rc::Rc};

use num::ToPrimitive;

use crate::{RuntimeError, Value, environment::{Environment, features::FeatureBuilder}, module::{Module, RuntimeModule}, procedures::{Procedure, RuntimeProcedure}};

use otr_core::{error::Result, r#struct::Struct, r#type::Type};

pub(crate) struct StringsFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl StringsFeatureBuilder {
    pub(crate) fn new_boxed() -> Box<dyn FeatureBuilder> {
        Box::new(Self { })
    }
}

impl FeatureBuilder for StringsFeatureBuilder {
    fn add_arg(&mut self, _arg_ident: &dyn AsRef<str>, _arg_value: &dyn AsRef<str>) -> Result<()> {
        Err(RuntimeError::Unknown { message: "Feature arguments not supported!".into() }.boxed())
    }

    fn build(&mut self) -> Result<RuntimeModule<'static>> {
        Ok(RuntimeModule::Abstract(Box::new(StringsFeature)))
    }
}

#[derive(Debug)]
struct StringsFeature;

impl Module for StringsFeature {
    fn get_procedure(
        &'_ self,
        identifier: &str,
        _private_access: bool,
    ) -> Result<RuntimeProcedure<'_>> {
        match identifier as &str {
            "length" => Ok(RuntimeProcedure::AbstractRef(&StringLengthProcdure)),
            "split" => Ok(RuntimeProcedure::AbstractRef(&StringSplitProcedure)),
            "toString" => Ok(RuntimeProcedure::AbstractRef(&ToStringProcedure)),
            "display" => Ok(RuntimeProcedure::AbstractRef(&DisplayProcedure)),
            "toCharArray" => Ok(RuntimeProcedure::AbstractRef(&StringToCharArrayProcedure)),
            "fromBytes" => Ok(RuntimeProcedure::AbstractRef(&FromBytesProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &str,
        procedure_identifier: &str,
        _private_access: bool,
    ) -> Result<RuntimeProcedure<'_>> {
        Err(RuntimeError::AssociatedProcedureNotDefined {
            procedure_identifier: procedure_identifier.to_string(),
            struct_identifier: struct_identifier.to_string()
        }.boxed())
    }

    fn get_struct(&self, identifier: &str, _private_access: bool) -> Result<Struct> {
        Err(RuntimeError::StructNotDefined {
            struct_identifier: identifier.to_string()
        }.boxed())
    }
}

#[derive(Debug)]
pub(crate) struct StringLengthProcdure;

impl Procedure for StringLengthProcdure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let str = arguments.first().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "string".into(),
            }
            .boxed(),
        )?;

        match str {
            Value::String(str) => Ok(Value::Integer(str.len() as i64)),

            other => Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: other.get_type_id(),
            }
            .boxed()),
        }
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct StringToCharArrayProcedure;

impl Procedure for StringToCharArrayProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let str = arguments.first().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "string".into(),
            }
            .boxed(),
        )?;

        match str {
            Value::String(str) => Ok(Value::Array(Rc::new(RefCell::new(Some(
                str.chars().map(Value::Char).collect::<Vec<Value>>().into_boxed_slice()
            ))))),

            other => Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: other.get_type_id(),
            }
            .boxed()),
        }
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct StringSplitProcedure;

impl Procedure for StringSplitProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let str = arguments.first().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "string".into(),
            }
            .boxed(),
        )?;
        let str = if let Value::String(str) = str {
            str
        } else {
            return Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: str.get_type_id(),
            }
            .boxed());
        };

        let pattern = arguments.get(1).ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "pattern".into(),
            }
            .boxed(),
        )?;
        let pattern = if let Value::String(pattern) = pattern {
            pattern
        } else {
            return Err(RuntimeError::TypeMismatch {
                expected: Type::String,
                found: pattern.get_type_id(),
            }
            .boxed());
        };

        Ok(Value::Array(Rc::new(RefCell::new(Some(
            str.split(pattern)
                .map(|part| Value::String(part.into()))
                .collect::<Vec<Value>>()
                .into_boxed_slice(),
        )))))
    }
    
    fn get_num_args(&self) -> usize {
        2
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct ToStringProcedure;

impl Procedure for ToStringProcedure {
    fn call(
        &self,
        _environment: Environment,
        mut arguments: Vec<Value>,
    ) -> Result<Value> {
        let value = arguments.pop().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "value".into(),
            }
            .boxed(),
        )?;

        let err = || -> Result<Value> {
            Err(
                RuntimeError::Unknown { message: "Only arrays of chars and strings can be joined to a string".into() }.boxed()
            )
        };

        let s = match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s,
            Value::Char(c) => c.to_string(),
            Value::Array(rc) => {
                let ref_cell = rc.borrow();

                let values = ref_cell.as_ref().ok_or_else(|| RuntimeError::UseOfMovedValue.boxed())?;

                let mut s = String::new();

                for item in values {
                    match item {
                        Value::Char(c) => {
                            s.push(*c);
                        }
                        Value::String(st) => {
                            s += st;
                        }
                        _ => {
                            return Err(
                                RuntimeError::Unknown { message: "Only arrays of chars and strings can be joined to a string".into() }.boxed()
                            );
                        }
                    }
                }

                s
            },
            Value::ArrayRef(weak) => {
                let rc = weak.upgrade().ok_or_else(|| RuntimeError::UseOfDroppedValue.boxed())?;

                let ref_cell = rc.borrow();

                let values = ref_cell.as_ref().ok_or_else(|| RuntimeError::UseOfMovedValue.boxed())?;

                let mut s = String::new();

                for item in values {
                    match item {
                        Value::Char(c) => {
                            s.push(*c);
                        }
                        Value::String(st) => {
                            s += st;
                        }
                        _ => {
                            return Err(
                                RuntimeError::Unknown { message: "Only arrays of chars and strings can be joined to a string".into() }.boxed()
                            );
                        }
                    }
                }

                s
            },
            _ => {
                return err();
            }
        };

        Ok(Value::String(s))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct DisplayProcedure;

impl Procedure for DisplayProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let value = arguments.first().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "value".into(),
            }
            .boxed(),
        )?;

        Ok(Value::String(value.to_string()))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct FromBytesProcedure;

impl Procedure for FromBytesProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let value = arguments.first().ok_or(
            RuntimeError::NoSuchVariable {
                variable_identifier: "value".into(),
            }
            .boxed(),
        )?;

        let bytes = match value {
            Value::Array(rc) => {
                let ref_cell = rc.borrow();

                let arr = ref_cell.as_ref().ok_or_else(|| RuntimeError::UseOfMovedValue.boxed())?;

                let mut bytes = Vec::with_capacity(arr.len());
                for (index, item) in arr.into_iter().enumerate() {
                    if let Value::Integer(byte) = item {
                        bytes.push(
                            byte.to_u8().ok_or(
                                RuntimeError::Unknown {
                                    message: format!(
                                        "Element of array at index {index} is not a valid byte!"
                                    ),
                                }
                                .boxed(),
                            )?,
                        );
                    };
                }
                bytes
            }
            Value::ArrayRef(weak) => {
                let rc = weak.upgrade().ok_or_else(|| RuntimeError::UseOfDroppedValue.boxed())?;

                let ref_cell = rc.borrow();

                let arr = ref_cell.as_ref().ok_or_else(|| RuntimeError::UseOfMovedValue.boxed())?;

                let mut bytes = Vec::with_capacity(arr.len());
                for (index, item) in arr.into_iter().enumerate() {
                    if let Value::Integer(byte) = item {
                        bytes.push(
                            byte.to_u8().ok_or(
                                RuntimeError::Unknown {
                                    message: format!(
                                        "Element of array at index {index} is not a valid byte!"
                                    ),
                                }
                                .boxed(),
                            )?,
                        );
                    };
                }
                bytes
            }
            _ => {
                return Err(RuntimeError::TypeMismatch {
                    expected: Type::Array,
                    found: value.get_type_id(),
                }
                .boxed());
            }
        };

        Ok(Value::String(String::from_utf8(bytes).map_err(|err| {
            RuntimeError::Unknown {
                message: err.to_string(),
            }
            .boxed()
        })?))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

