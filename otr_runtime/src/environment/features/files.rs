use std::{cell::RefCell, fs, rc::Rc};

use num::ToPrimitive;

use otr_core::{module::ModuleAddress, r#struct::Struct, r#type::Type, value::Value, error::Result};
use crate::{environment::{Environment, features::FeatureBuilder}, error::RuntimeError, module::{Module, RuntimeModule}, procedures::{Procedure, RuntimeProcedure}};

pub(crate) struct FilesFeatureBuilder {
    //TODO: Add support for feature arguments
}

impl FilesFeatureBuilder {
    pub(crate) fn new_boxed() -> Box<dyn FeatureBuilder> {
        Box::new(Self { })
    }
}

impl FeatureBuilder for FilesFeatureBuilder {
    fn add_arg(&mut self, _arg_ident: &dyn AsRef<str>, _arg_value: &dyn AsRef<str>) -> Result<()> {
        Err(RuntimeError::Unknown { message: "Feature arguments not supported!".into() }.boxed())
    }

    fn build(&mut self) -> Result<RuntimeModule<'static>> {
        Ok(RuntimeModule::Abstract(Box::new(FilesFeature)))
    }
}

#[derive(Debug)]
struct FilesFeature;

impl Module for FilesFeature {
    fn get_procedure(
        &'_ self,
        identifier: &str,
        _private_access: bool,
    ) -> Result<crate::procedures::RuntimeProcedure<'_>> {
        match identifier as &str {
            "read" => Ok(RuntimeProcedure::AbstractRef(&FSReadProcedure)),
            "write" => Ok(RuntimeProcedure::AbstractRef(&FSWriteProcedure)),
            "exists" => Ok(RuntimeProcedure::AbstractRef(&FSExistsProcedure)),
            "listDir" => Ok(RuntimeProcedure::AbstractRef(&FSListDirProcedure)),
            "removeFile" => Ok(RuntimeProcedure::AbstractRef(&FSRemoveFileProcedure)),
            "removeDir" => Ok(RuntimeProcedure::AbstractRef(&FSRemoveDirProcedure)),

            unknown => Err(RuntimeError::ProcedureNotDefined { procedure_identifier: unknown.to_string() }.boxed())
        }
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &str,
        procedure_identifier: &str,
        _private_access: bool,
    ) -> Result<crate::procedures::RuntimeProcedure<'_>> {
        Err(RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: procedure_identifier.to_string(), struct_identifier: struct_identifier.to_string() }.boxed())
    }

    fn get_struct(&self, identifier: &str, _private_access: bool) -> Result<Struct> {
        Err(RuntimeError::StructNotDefined { struct_identifier: identifier.to_string() }.boxed())
    }
}

fn directory(path: String) -> Struct {
    let mut dir = Struct::new(ModuleAddress::new("Files".into(), "Directory".into()));
    let _ = dir
        .get_members_mut()
        .insert("path".into(), Value::String(path), true);
    dir
}

fn file(path: String) -> Struct {
    let mut f = Struct::new(ModuleAddress::new("Files".into(), "File".into()));
    let _ = f
        .get_members_mut()
        .insert("path".into(), Value::String(path), true);
    f
}

fn get_path(arguments: &[Value]) -> Result<&String> {
    let path = arguments.first().ok_or(
        RuntimeError::NoSuchVariable {
            variable_identifier: "path".into(),
        }
        .boxed(),
    )?;
    if let Value::String(s) = path {
        Ok(s)
    } else {
        Err(RuntimeError::TypeMismatch {
            expected: Type::String,
            found: path.get_type_id(),
        }
        .boxed())
    }
}

#[derive(Debug)]
pub(crate) struct FSReadProcedure;

impl Procedure for FSReadProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let path = get_path(&arguments)?;

        let bytes = fs::read(path).map_err(|err| {
            RuntimeError::Unknown {
                message: err.to_string(),
            }
            .boxed()
        })?;

        Ok(Value::Array(Rc::new(RefCell::new(Some(
            bytes
                .into_iter()
                .map(|byte| Value::Integer(byte as i64))
                .collect::<Vec<Value>>()
                .into_boxed_slice(),
        )))))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct FSWriteProcedure;

impl Procedure for FSWriteProcedure {
    fn call(
        &self,
        _environment: Environment,
        mut arguments: Vec<Value>,
    ) -> Result<Value> {
        if arguments.len() < 2 {
            return Err(RuntimeError::NoSuchVariable {
                variable_identifier: "data".into(),
            }
            .boxed());
        }
        let data = arguments.swap_remove(1);

        let path = get_path(&arguments)?;

        let bytes = match data {
            Value::String(s) => s.into_bytes(),
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
            other => {
                return Err(RuntimeError::Unknown {
                    message: format!(
                        "Cannot write value of type {} to file!",
                        other.get_type_id()
                    ),
                }
                .boxed());
            }
        };

        fs::write(path, bytes)
            .map_err(|err| {
                RuntimeError::Unknown {
                    message: err.to_string(),
                }
                .boxed()
            })
            .map(|_| Value::Null)
    }
    
    fn get_num_args(&self) -> usize {
        2
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct FSExistsProcedure;

impl Procedure for FSExistsProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let path = get_path(&arguments)?;

        fs::exists(path)
            .map_err(|err| {
                RuntimeError::Unknown {
                    message: err.to_string(),
                }
                .boxed()
            })
            .map(Value::Bool)
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct FSListDirProcedure;

impl Procedure for FSListDirProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let path = get_path(&arguments)?;

        let entries = fs::read_dir(path).map_err(|err| {
            RuntimeError::Unknown {
                message: err.to_string(),
            }
            .boxed()
        })?;

        let mut out = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|err| {
                RuntimeError::Unknown {
                    message: err.to_string(),
                }
                .boxed()
            })?;

            let path = entry.path();
            if path.is_dir() {
                out.push(Value::Struct(Rc::new(RefCell::new(Some(directory(
                    path.to_str().unwrap().to_owned(),
                ))))))
            } else {
                out.push(Value::Struct(Rc::new(RefCell::new(Some(file(
                    path.to_str().unwrap().to_owned(),
                ))))))
            }
        }

        Ok(Value::Array(Rc::new(RefCell::new(Some(out.into_boxed_slice())))))
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct FSRemoveFileProcedure;

impl Procedure for FSRemoveFileProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let path = get_path(&arguments)?;

        fs::remove_file(path)
            .map_err(|err| {
                RuntimeError::Unknown {
                    message: err.to_string(),
                }
                .boxed()
            })
            .map(|_| Value::Null)
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub(crate) struct FSRemoveDirProcedure;

impl Procedure for FSRemoveDirProcedure {
    fn call(
        &self,
        _environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let path = get_path(&arguments)?;

        fs::remove_dir(path)
            .map_err(|err| {
                RuntimeError::Unknown {
                    message: err.to_string(),
                }
                .boxed()
            })
            .map(|_| Value::Null)
    }
    
    fn get_num_args(&self) -> usize {
        1
    }
    
    fn get_stack_size(&self) -> usize {
        0
    }
}
