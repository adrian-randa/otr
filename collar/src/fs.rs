use otr_config::Version;
use otr_core::Result;

use std::{env::current_dir, fs::{self, Permissions}, os::unix::fs::PermissionsExt, path::{Path, PathBuf}};

use crate::{catch, error::CollarError};

pub fn get_current_dir() -> Result<PathBuf> {
    current_dir().map_err(
        |err| CollarError::new(format!("Could not get current directory: {err}")).boxed()
    )
}

pub fn ensure_dir_exists(dir: &Path) -> Result<bool> {
    if std::fs::exists(dir).map_err(
        |err| CollarError::new(format!("Filesystem error: {err}")).boxed()
    )? {
        Ok(true)
    } else {
        std::fs::create_dir(dir).map_err(
            |err| CollarError::new(format!("Filesystem error: {err}")).boxed()
        )?;
        Ok(false)
    }
}

fn get_home_dir() -> Result<PathBuf> {
    let home_dir = std::env::home_dir()
        .ok_or(CollarError::new("Could not get your home directory!").boxed())?;

    if home_dir.as_os_str().is_empty() {
        Err(CollarError::new(
            "Could not get your home directory!"
        ).boxed())
    } else {
        Ok(home_dir)
    }
}

pub fn get_collar_dir() -> Result<PathBuf> {
    let home_dir = get_home_dir()?;

    let collar_dir = home_dir.join(".collar");

    ensure_dir_exists(&collar_dir)?;

    Ok(collar_dir)
}

pub fn get_otrc_dir() -> Result<PathBuf> {
    let collar_dir = get_collar_dir()?;

    let bin_path = collar_dir.join("bin");
    let otrc_dir = bin_path.join("otrc");

    ensure_dir_exists(&bin_path)?;
    ensure_dir_exists(&otrc_dir)?;

    Ok(otrc_dir)
}

pub fn get_otrrun_dir() -> Result<PathBuf> {
    let collar_dir = get_collar_dir()?;

    let bin_path = collar_dir.join("bin");
    let otrrun_dir = bin_path.join("otrrun");

    ensure_dir_exists(&bin_path)?;
    ensure_dir_exists(&otrrun_dir)?;

    Ok(otrrun_dir)
}

pub fn get_otrc_bin_path(version: Option<Version>) -> Result<PathBuf> {
    
    let version = match version {
        Some(version) => version,
        None => get_installed_versions()?.latest_otrc().ok_or(
            CollarError::new("No version of 'otrc' installed!").boxed()
        )?
    };

    let path = get_otrc_dir()?.join(version.to_string());

    Ok(path)
}

pub fn get_otrrun_bin_path(version: Option<Version>) -> Result<PathBuf> {
    
    let version = match version {
        Some(version) => version,
        None => get_installed_versions()?.latest_otrrun().ok_or(
            CollarError::new("No version of 'otrrun' installed!").boxed()
        )?
    };

    let path = get_otrrun_dir()?.join(version.to_string());

    Ok(path)
}

pub fn install_otrc(version: Version, bytes: impl AsRef<[u8]>) -> Result<()> {
    let otrc_bin_path = get_otrc_bin_path(Some(version))?;

    catch(fs::write(&otrc_bin_path, bytes), "Could not write 'otrc' binary")?;

    if cfg!(unix) {
        let file = catch(fs::File::open(&otrc_bin_path), "Could not set execute permissions")?;
    
        let perms = Permissions::from_mode(0o744);

        catch(file.set_permissions(perms), "Could not set execute permissions")?;
    }

    Ok(())
}

pub fn install_otrrun(version: Version, bytes: impl AsRef<[u8]>) -> Result<()> {
    let otrrun_bin_path = get_otrrun_bin_path(Some(version))?;

    catch(fs::write(&otrrun_bin_path, bytes), "Could not write 'otrrun' binary")?;

    if cfg!(unix) {
        let file = catch(fs::File::open(&otrrun_bin_path), "Could not set execute permissions")?;
    
        let perms = Permissions::from_mode(0o744);

        catch(file.set_permissions(perms), "Could not set execute permissions")?;
    }

    Ok(())
}

pub struct InstalledVersions {
    pub otrc: Vec<Version>,
    pub otrrun: Vec<Version>,
}

impl InstalledVersions {
    pub fn latest_otrc(&self) -> Option<Version> {
        self.otrc.iter().max().map(|v| v.to_owned())
    }

    pub fn latest_otrrun(&self) -> Option<Version> {
        self.otrrun.iter().max().map(|v| v.to_owned())
    }
}

pub fn get_installed_versions() -> Result<InstalledVersions> {

    let collar_dir = get_collar_dir()?;
    let bin_dir = collar_dir.join("bin");
    let otrrun_dir = bin_dir.join("otrrun");
    let otrc_dir = bin_dir.join("otrc");

    ensure_dir_exists(&bin_dir)?;
    ensure_dir_exists(&otrrun_dir)?;
    ensure_dir_exists(&otrc_dir)?;

    Ok(InstalledVersions {
        otrc: get_versions_in_dir(&otrc_dir)?,
        otrrun: get_versions_in_dir(&otrrun_dir)?,
    })
}

fn get_versions_in_dir(dir: &Path) -> Result<Vec<Version>> {
    
    let mut versions = Vec::new();

    for entry in std::fs::read_dir(dir).map_err(
        |err| CollarError::new(format!("Filesystem error: {err}")).boxed()
    )? {
        if entry.is_err() { continue }
        let entry = entry.unwrap();

        if !entry.path().is_file() { continue }

        let name = entry.file_name();
        let name_str = name.to_str();
        if name_str.is_none() { continue }
        let name = name_str.unwrap();

        if let Ok(version) = Version::try_from(name) {
            versions.push(version);
        }
    }

    Ok(versions)
}

pub fn uninstall_otrc(version: Version) -> Result<()> {
    let path = get_otrc_bin_path(Some(version))?;

    if !catch(fs::exists(&path), "Could not locate 'otrc' binary")? {
        return Err(CollarError::new("This version of 'otrc' is not installed!").boxed());
    }

    catch(fs::remove_file(&path), "Could not remove 'otrc' binary")
}

pub fn uninstall_otrrun(version: Version) -> Result<()> {
    let path = get_otrrun_bin_path(Some(version))?;

    if !catch(fs::exists(&path), "Could not locate 'otrrun' binary")? {
        return Err(CollarError::new("This version of 'otrrun' is not installed!").boxed());
    }

    catch(fs::remove_file(&path), "Could not remove 'otrrun' binary")
}