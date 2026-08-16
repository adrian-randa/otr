use std::{collections::HashSet, env, fs, path::{Path, PathBuf}};

use otr_config::Features;
use otr_core::{Result, SystemError, module::{CompiledModule, ImportAddress}};
use otr_runtime::{EnvironmentBuilder, RuntimeObject, environment::Environment};

pub fn get_current_dir() -> Result<PathBuf> {
    env::current_dir().map_err(|err| SystemError::new(
        format!("Could not get current directory: {err}")
    ).boxed())
}

pub fn run_script(object_path: &Path) -> Result<()> {
    if !object_path.is_file() {
        return Err(SystemError::new("Supplied object path does not point to a file!".into()).boxed());
    }

    let path = if object_path.is_relative() {
        get_current_dir()?.join(object_path)
    } else {
        object_path.to_path_buf()
    };

    let root_path = path.parent().ok_or(SystemError::new(
        "Could not extract parent directory for the supplied script!".into()
    ).boxed())?;
    let module_ident = path.file_stem()
        .ok_or(SystemError::new(
            "Could not extract module ident for the supplied script!".into()
        ).boxed())?
        .to_str()
        .ok_or(SystemError::new("Module ident is not a valid string!".into()).boxed())?
        .to_string();

    let module = read_module(
        root_path,
        ImportAddress {
            module_id: module_ident.clone(),
            path: None,
            alias: None,
        }
    )?;

    let runtime_object = RuntimeObject::builder(Environment::default())
        .with_root(module, module_ident)
        .build();

    runtime_object.execute()?;

    Ok(())
}

pub fn build_environment_from_features(features: Features) -> Result<Environment<'static>> {
    let mut environment = EnvironmentBuilder::new();

    for feature in features {
        let mut feature_environment = environment.with_feature(feature.get_feature_ident())?;

        for (arg_ident, arg_value) in feature.get_config() {
            feature_environment = feature_environment.with_arg(arg_ident, arg_value)?;
        }

        environment = feature_environment.finalize_feature()?;
    }

    Ok(environment.build())
}

pub fn build_runtime_object(root_path: &Path, root_module_address: ImportAddress, base_environment: Environment<'static>) -> Result<RuntimeObject<'static>> {
    
    let builder = RuntimeObject::builder(base_environment);

    let mut load_queue = Vec::new();
    let mut loaded = HashSet::new();
    loaded.insert(root_module_address.clone());

    let root_module_ident = root_module_address.module_id.clone();
    let root_module = read_module(root_path, root_module_address)?;

    load_queue.extend_from_slice(root_module.get_dependencies());

    let mut builder = builder.with_root(root_module, root_module_ident);

    while let Some(address) = load_queue.pop() {
        if !loaded.insert(address.clone()) {
            continue;
        }
        
        let module_ident = address.module_id.clone();
        let alias = address.alias.clone();
        let module = read_module(root_path, address)?;

        load_queue.extend_from_slice(module.get_dependencies());

        builder = builder.with_module(module, alias.unwrap_or(module_ident))
    }

    Ok(builder.build())
}

pub fn read_module(root_file_path: &Path, address: ImportAddress) -> Result<CompiledModule> {
    let module_file_name = address.to_flat_string();

    let bytes = fs::read(root_file_path.join(module_file_name).with_extension("oco"))
        .map_err(|err| SystemError::new(
            format!("Could not read module: {err}")
        ).boxed())?;
    
    serde_cbor::from_slice(&bytes)
        .map_err(|err| SystemError::new(
            format!("Could not deserialize module: {err}")
        ).boxed())
}