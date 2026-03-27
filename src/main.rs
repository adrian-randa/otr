use std::{env, fs, path::PathBuf};

use otr::{compiler::{Compiler, source_file_reader::{ImportAddress, SourceFileReader}}, config::{self, Features, ProjectConfiguration}, error::system_error::SystemError, lexer::Tokenizer, runtime::{RuntimeObject, environment::{Environment, environment_builder::EnvironmentBuilder}}};
use otr::error::Result;

fn main() {
    //run();
    match get_project_config() {
        Ok(conf) => println!("{:#?}", conf),
        Err(err) => println!("{err}"),
    }
}

fn get_project_config() -> Result<ProjectConfiguration> {
    const CONFIG_PATH: &'static str = "otr_config.toml";

    let config_string = fs::read_to_string(env::current_dir().unwrap().join(CONFIG_PATH)).map_err(|err| SystemError::new(
        format!("Could not read project configuration file: {}", err)
    ).boxed())?;

    let config = toml::from_str(&config_string).map_err(|err| SystemError::new(
        format!("Could not parse project configuration file: {}", err)
    ).boxed())?;

    Ok(config)
}

fn build_environment_with_features(features: Features) -> Result<Environment<'static>> {
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

fn run(root_file_path: PathBuf, root_module: String) -> Result<()> {
    let compiler = Compiler::new(
        Tokenizer::default(),
        root_file_path,
        root_module
    )?;

    let compiled_object = compiler.compile()?;
    
    let runtime_object = RuntimeObject::from(compiled_object);

    runtime_object.execute().map(|_| ())
}