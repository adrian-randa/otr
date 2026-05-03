use std::path::{Path, PathBuf};

use otr_config::{FeatureConfig, Features, ProjectConfiguration, ProjectType, Version, get_project_config};
use otr_core::Result;

use crate::{cmd::{compile_project, run_project}, error::CollarError};

const MAIN_OTR_TEMPLATE: &'static str = include_str!("../template/Main.otr");
const LIBMAIN_OTR_TEMPLATE: &'static str = include_str!("../template/LibMain.otr");

pub fn catch<T, E: std::fmt::Display>(result: std::result::Result<T, E>, prefix: impl AsRef<str>) -> Result<T> {
    let prefix = prefix.as_ref();

    result.map_err(
        |err| CollarError::new(format!("{prefix}: {err}")).boxed()
    )
}

pub fn config_path(root: &Path) -> PathBuf {
    root.join("otr_config").with_extension("toml")
}

pub fn new_executable(root: &Path, name: String, otr_version: Version) -> Result<()> {
    let config_path = config_path(root);

    if std::fs::exists(&config_path).map_err(
        |err| CollarError::new(format!("Filesystem error: {err}")).boxed()
    )? {
        return Err(CollarError::new("A project is already initialized in this directory!").boxed());
    }

    let template_config = ProjectConfiguration {
        project: ProjectType::Executable,
        name,
        otr_version,
        root_module: "Main".into(),
        features: Features(
            [
                ("Debug".into(), FeatureConfig([].into()))
            ].into()
        ),
    };

    let template_str = toml::to_string(&template_config).unwrap();

    std::fs::write(&config_path, template_str)
        .map_err(|err| CollarError::new(format!("Could not write template config file: {err}")).boxed())?;

    std::fs::write(root.join("Main").with_extension("otr"), MAIN_OTR_TEMPLATE)
        .map_err(|err| CollarError::new(format!("Could not write template 'Main.otr' file: {err}")).boxed())?;

    Ok(())
}

pub fn new_library(root: &Path, name: String, otr_version: Version) -> Result<()> {
    let config_path = config_path(root);

    if std::fs::exists(&config_path).map_err(
        |err| CollarError::new(format!("Filesystem error: {err}")).boxed()
    )? {
        return Err(CollarError::new("A project is already initialized in this directory!").boxed());
    }

    
    let template_config = ProjectConfiguration {
        project: ProjectType::Library,
        name: name.clone(),
        root_module: name.clone(),
        otr_version,
        features: Features(
            [
                ("Debug".into(), FeatureConfig([].into()))
                ].into()
            ),
        };
        
        let template_str = toml::to_string(&template_config).unwrap();

    std::fs::write(&config_path, template_str)
        .map_err(|err| CollarError::new(format!("Could not write template config file: {err}")).boxed())?;

    let root_module = LIBMAIN_OTR_TEMPLATE.replace("<MODNAME>", &name);
    
    std::fs::write(root.join(&name).with_extension("otr"), root_module)
    .map_err(|err| CollarError::new(format!("Could not write template '{name}.otr' file: {err}")).boxed())?;

    Ok(())
}

pub fn get_config(root: &Path) -> Result<ProjectConfiguration> {
    let config_path = config_path(root);

    get_project_config(config_path)
}

pub fn compile_and_run_project(root: &Path, root_module: impl AsRef<str>) -> Result<()> {
    compile_project(root, root_module)?;
    run_project(root)?;

    Ok(())
}

pub mod error;
pub mod fs;
pub mod cmd;
pub mod net;