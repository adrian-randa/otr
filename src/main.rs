use std::{env, fs, path::PathBuf};

use otr::{compiler::Compiler, config::{self, Features, ProjectConfiguration}, error::system_error::SystemError, lexer::Tokenizer, runtime::{RuntimeObject, environment::{Environment, environment_builder::EnvironmentBuilder}}};
use otr::error::Result;

fn main() {
    //let config = get_project_config().unwrap();

    //let root_module = config.get_root_module();

    //run(env::current_dir().unwrap(), root_module.into()).unwrap();
    /* match get_project_config() {
        Ok(conf) => println!("{:#?}", conf),
        Err(err) => println!("{err}"),
    } */
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