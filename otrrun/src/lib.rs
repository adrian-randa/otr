use std::{collections::HashSet, env, fs, path::{Path, PathBuf}};

use otr_config::{Features, GlobalConfiguration};
use otr_core::{Result, SystemError, module::{CompiledModule, ImportAddress}};
use otr_runtime::{EnvironmentBuilder, RuntimeObject, environment::Environment, external::ExternalModule};

pub enum Module {
    Compiled(CompiledModule),
    External(ExternalModule),
}

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
        },
        None
    )?;

    let module = if let Module::Compiled(module) = module {
        module
    } else {
        return Err(SystemError::new("Cannot run external module as script!".into()).boxed());
    };

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

pub fn build_runtime_object(root_path: &Path, root_module_address: ImportAddress, base_environment: Environment<'static>, global_configuration: Option<GlobalConfiguration>) -> Result<RuntimeObject<'static>> {
    
    let builder = RuntimeObject::builder(base_environment);

    let mut load_queue = Vec::new();
    let mut loaded = HashSet::new();
    loaded.insert(root_module_address.clone());

    let root_module_ident = root_module_address.module_id.clone();
    let root_module = if let Module::Compiled(module) = read_module(root_path, root_module_address, global_configuration.as_ref())? {
        module
    } else {
        return Err(SystemError::new("Root module must not be an external module!".into()).boxed());
    };

    load_queue.extend_from_slice(root_module.get_dependencies());

    let mut builder = builder.with_root(root_module, root_module_ident);

    while let Some(address) = load_queue.pop() {
        if !loaded.insert(address.clone()) {
            continue;
        }
        
        let module_ident = address.module_id.clone();
        let alias = address.alias.clone();
        let module = read_module(root_path, address, global_configuration.as_ref())?;

        match module {
            Module::Compiled(compiled_module) => {
                load_queue.extend_from_slice(compiled_module.get_dependencies());
        
                builder = builder.with_compiled_module(compiled_module, alias.unwrap_or(module_ident))
            },
            Module::External(external_module) => {
                builder = builder.with_external_module(external_module, alias.unwrap_or(module_ident))
            },
        }
    }

    Ok(builder.build())
}

pub fn read_module(root_file_path: &Path, address: ImportAddress, global_configuration: Option<&GlobalConfiguration>) -> Result<Module> {
    let module_file_name = address.clone().to_flat_string();
    
    if fs::exists(root_file_path.join(&module_file_name).with_extension("oco"))
        .map_err(|err| SystemError::new(
            format!("Could not read module: {err}")
        ).boxed())?
    {
        read_compiled_module(root_file_path, address).map(Module::Compiled)
    } else {
        if fs::exists(root_file_path.join(&module_file_name).with_extension("oem"))
            .map_err(|err| SystemError::new(
                format!("Could not read module: {err}")
            ).boxed())?
        {
            read_external_module(root_file_path, address, global_configuration).map(Module::External)
        } else {
            Err(SystemError::new(format!("Module '{}' not found!", address.module_id)).boxed())
        }
    }
}

fn read_compiled_module(root_file_path: &Path, address: ImportAddress) -> Result<CompiledModule> {
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

fn read_external_module(root_file_path: &Path, address: ImportAddress, global_configuration: Option<&GlobalConfiguration>) -> Result<ExternalModule> {
    let module_file_name = address.clone().to_flat_string();

    let bytes = fs::read(root_file_path.join(module_file_name).with_extension("oem"))
        .map_err(|err| SystemError::new(
            format!("Could not read module: {err}")
        ).boxed())?;

    let module: otr_ffi::external::ExternalModule = serde_cbor::from_slice(&bytes)
        .map_err(|err| SystemError::new(
            format!("Could not deserialize module: {err}")
        ).boxed())?;
    
    let library_file_path = resolve_library_path(root_file_path, address, global_configuration)?;

    let library = unsafe { libloading::Library::new(library_file_path) }
        .map_err(|err| SystemError::new(format!("Library could not be loaded: {err}")).boxed())?;

    let mut binded_module = ExternalModule::new(module.clone());

    for symbol in module.functions {
        let function = unsafe { library.get(&symbol.0) }
            .map_err(|err| SystemError::new(format!("Could not bind to member {}: {err}", &symbol.0)).boxed())?;

        binded_module.insert_binding(symbol.0, *function)?;
    }

    Ok(binded_module)
}

fn resolve_library_path(root_file_path: &Path, address: ImportAddress, global_configuration: Option<&GlobalConfiguration>) -> Result<PathBuf> {

    if let Some(path) = &address.path {
        let mut path = path as &str;

        if path.starts_with("@") {

            let root;
            if let Some(p) = path[1..].split_once("/") {
                root = p.0;
                path = p.1;
            } else {
                root = &path[1..];
                path = "";
            }

            let root = global_configuration
                .ok_or(SystemError::new(format!("Tried to resolve root '{root}', but no global configuration has been supplied")).boxed())?
                .try_resolve_root(root).ok_or(
                    SystemError::new(format!("Could not find root for '{root}'")).boxed()
                )?;
            
            let extension = if cfg!(windows) {
                "dll"
            } else {
                "so"
            };

            return Ok(root.join(path).join(address.module_id).with_extension(extension))
        }
    }

    Ok(root_file_path
        .join(address.path.as_ref().map(|r| r as &str).unwrap_or(""))
        .join(address.module_id)
        .with_extension("otr"))
}