use std::{fs, path::{Path, PathBuf}};

use otr_compiler::{Compiler, CompilerEnvironment, Fragmenter, lexer::Tokenizer};
use otr_config::GlobalConfiguration;
use otr_core::{SystemError, error::Result, module::{CompiledModule, ImportAddress}};


pub fn compile_and_write_dependency_tree(root_file_path: &Path, root_module_name: String, global_configuration: Option<GlobalConfiguration>) -> Result<()> {
    let mut environment = CompilerEnvironment::new();

    environment.push_file_to_queue(ImportAddress {
        module_id: root_module_name,
        path: None
    });

    while let Some(address) = environment.get_next_file() {
        let source = read_source_file(root_file_path.to_path_buf(), address.clone(), global_configuration.as_ref())?;

        let compiled_module = compile_single_file(source, &mut environment)?;

        write_compiled_file(root_file_path, address, &compiled_module)?;
    }

    Ok(())
}

pub fn compile_single_file(source: String, environment: &mut CompilerEnvironment) -> Result<CompiledModule> {
    let fragments = Fragmenter::fragment(&source)?;
    let tokens = Tokenizer::default().tokenize(fragments)?;

    let compiler = Compiler::new();

    compiler.compile(tokens.into_iter(), environment)
}

pub fn read_source_file(root_file_path: PathBuf, address: ImportAddress, global_configuration: Option<&GlobalConfiguration>) -> Result<String> {
    let file_path = resolve_import_address(&root_file_path, address, global_configuration)?;

    fs::read_to_string(file_path)
        .map_err(|err| SystemError::new(format!("Could not read source file: {}", err)).boxed())
}

pub fn write_compiled_file(root_file_path: &Path, address: ImportAddress, module: &CompiledModule) -> Result<()> {
    let output_dir_path = root_file_path.join("compiled");

    let output_dir_exists = fs::exists(&output_dir_path).map_err(|err| SystemError::new(
    format!("Could not locate output directory: {err}")
    ).boxed())?;

    if !output_dir_exists {
        fs::create_dir(&output_dir_path)
            .map_err(|err| 
                SystemError::new(format!("Could not create output directory: {err}")).boxed()
            )?;
    }

    let bytes = serde_cbor::to_vec(module).map_err(|err| 
        SystemError::new(format!("Could not serialize compiled module: {err}")).boxed()
    )?;

    let output_file_name = address.to_flat_string();
    
    let output_file_path = output_dir_path.join(output_file_name).with_extension("oco");

    fs::write(output_file_path, bytes).map_err(|err|
        SystemError::new(format!("Could not write compiled file: {err}")).boxed()
    )
}

fn resolve_import_address(root_file_path: &Path, address: ImportAddress, global_configuration: Option<&GlobalConfiguration>) -> Result<PathBuf> {

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

            return Ok(root.join(path).join(address.module_id).with_extension("otr"))
        }
    }

    Ok(root_file_path
        .join(address.path.as_ref().map(|r| r as &str).unwrap_or(""))
        .join(address.module_id)
        .with_extension("otr"))
}