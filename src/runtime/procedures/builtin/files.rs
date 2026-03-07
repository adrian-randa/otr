use std::{cell::RefCell, fs, rc::Rc};

use num::ToPrimitive;

use crate::{
    error::{runtime_error::RuntimeError, Result},
    runtime::{
        module::{CompiledModule, Module},
        procedures::Procedure,
        ModuleAddress, Struct, Type, Value,
    },
};

pub(crate) fn get_module() -> CompiledModule {
    let mut module = CompiledModule::default();

    module.insert_procedure("read".into(), Box::new(FSReadProcedure), true);
    module.insert_procedure("write".into(), Box::new(FSWriteProcedure), true);
    module.insert_procedure("exists".into(), Box::new(FSExistsProcedure), true);
    module.insert_procedure("listDir".into(), Box::new(FSListDirProcedure), true);
    module.insert_procedure("removeFile".into(), Box::new(FSRemoveFileProcedure), true);
    module.insert_procedure("removeDir".into(), Box::new(FSRemoveDirProcedure), true);

    module
}

fn directory(path: String) -> Struct {
    let mut dir = Struct::new(ModuleAddress::new("Files".into(), "Directory".into()));
    let _ = dir
        .get_members_mut()
        .insert_member("path".into(), Value::String(path), true);
    dir
}

fn file(path: String) -> Struct {
    let mut f = Struct::new(ModuleAddress::new("Files".into(), "File".into()));
    let _ = f
        .get_members_mut()
        .insert_member("path".into(), Value::String(path), true);
    f
}

fn get_path(arguments: &Vec<Value>) -> Result<&String> {
    let path = arguments.get(0).ok_or(
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
        _environment: crate::runtime::environment::Environment,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let path = get_path(&arguments)?;

        let bytes = fs::read(path).map_err(|err| {
            RuntimeError::Unknown {
                message: err.to_string(),
            }
            .boxed()
        })?;

        Ok(Value::Array(
            bytes
                .into_iter()
                .map(|byte| Value::Integer(byte as i64))
                .collect(),
        ))
    }
}

#[derive(Debug)]
pub(crate) struct FSWriteProcedure;

impl Procedure for FSWriteProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
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
            Value::Array(arr) => {
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
}

#[derive(Debug)]
pub(crate) struct FSExistsProcedure;

impl Procedure for FSExistsProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
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
            .map(|b| Value::Bool(b))
    }
}

#[derive(Debug)]
pub(crate) struct FSListDirProcedure;

impl Procedure for FSListDirProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
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

        Ok(Value::Array(out))
    }
}

#[derive(Debug)]
pub(crate) struct FSRemoveFileProcedure;

impl Procedure for FSRemoveFileProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
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
}

#[derive(Debug)]
pub(crate) struct FSRemoveDirProcedure;

impl Procedure for FSRemoveDirProcedure {
    fn call(
        &self,
        _environment: crate::runtime::environment::Environment,
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
}
