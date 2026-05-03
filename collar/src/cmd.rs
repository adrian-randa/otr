use std::io::Write;
use std::path::{Path, PathBuf};

use std::process::{Command, Stdio};

use otr_config::{Version, get_project_config};
use otr_core::Result;

use crate::catch;
use crate::fs::{get_otrc_bin_path, get_otrrun_bin_path};
use crate::{config_path, error::CollarError, fs::{get_installed_versions, get_otrc_dir, get_otrrun_dir}};

pub fn compile_project(root: &Path, root_module: impl AsRef<str>) -> Result<()> {
    let config = get_project_config(config_path(root))?;

    let version = config.otr_version;

    let otrc_bin_path = get_otrc_bin_path(Some(version))?;

    let output = Command::new(otrc_bin_path)
        .arg(root_module.as_ref())
        .output()
        .map_err(
            |err| CollarError::new(format!("Could not launch 'otrc' process: {err}")).boxed()
        )?;

    std::io::stdout().write_all(&output.stdout).map_err(
        |err| CollarError::new(format!("Could not display the output of 'otrc': {err}")).boxed()
    )
}

pub fn run_project(root: &Path) -> Result<()> {

    let config = get_project_config(config_path(root))?;

    let version = config.otr_version;

    let otrrun_bin_path = get_otrrun_bin_path(Some(version))?;

    let output = Command::new(otrrun_bin_path)
        .arg("project")
        .arg("--path").arg("./compiled")
        .stdout(Stdio::inherit())
        .output()
        .map_err(
            |err| CollarError::new(format!("Could not launch 'otrrun' process: {err}")).boxed()
        )?;

    fn write(buf: Vec<u8>) -> Result<()> {
        std::io::stdout().write_all(&buf).map_err(
            |err| CollarError::new(format!("Could not display the output of 'otrrun': {err}")).boxed()
        )
    }

    if output.status.success() {
        write(output.stdout)?;
    } else {
        write(output.stderr)?;
    }

    catch(std::io::stdout().flush(), "Could not display the output of 'otrrun'")?;

    Ok(())
}