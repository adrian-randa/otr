use std::{fs, path::{Path, PathBuf}, str::FromStr};

use otr_compiler::{Compiler, CompilerEnvironment, lexer::{Tokenizer, fragmenter::FragmentStream}};
use otr_core::{SystemError, error::Result, module::{CompiledModule, ImportAddress}};


pub fn compile_and_write_dependency_tree(root_file_path: &Path, root_module_name: String) -> Result<()> {
    let mut environment = CompilerEnvironment::new();

    environment.push_file_to_queue(ImportAddress {
        module_id: root_module_name,
        path: None
    });

    while let Some(address) = environment.get_next_file() {
        let source = read_source_file(root_file_path.to_path_buf(), address.clone())?;

        let compiled_module = compile_single_file(source, &mut environment)?;

        write_compiled_file(root_file_path, address, &compiled_module)?;
    }

    Ok(())
}

pub fn compile_single_file(source: String, environment: &mut CompilerEnvironment) -> Result<CompiledModule> {
    let fragments = FragmentStream::from_str(&source)?;
    let tokens = Tokenizer::default().tokenize(fragments)?;

    let compiler = Compiler::new();

    compiler.compile(tokens.into_iter(), environment)
}

pub fn read_source_file(root_file_path: PathBuf, address: ImportAddress) -> Result<String> {
    let file_path = root_file_path
            .join(address.path.as_ref().map(|r| r as &str).unwrap_or(""))
            .join(address.module_id)
            .with_extension("otr");

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