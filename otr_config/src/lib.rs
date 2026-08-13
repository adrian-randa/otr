use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use otr_core::error::{Error, system_error::SystemError};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(try_from = "&str")]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.patch.cmp(&other.patch)
    }
}

impl Version {
    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self { major, minor, patch }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl TryFrom<&str> for Version {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let err = || SystemError::new("Invalid version number!".into()).boxed();
        let mut iter = value.split(".");

        let major = iter.next().ok_or(err())?.parse().map_err(|_| err())?;
        let minor = iter.next().ok_or(err())?.parse().map_err(|_| err())?;
        let patch = iter.next().ok_or(err())?.parse().map_err(|_| err())?;

        if iter.next().is_some() {
            return Err(err());
        }

        Ok(Self { major, minor, patch })
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        format!("{}.{}.{}", self.major, self.minor, self.patch).serialize(serializer)
    }
}

pub struct Feature {
    pub feature_ident: String,
    pub config: Vec<(String, String)>
}

impl Feature {
    pub fn get_feature_ident(&self) -> &String {
        &self.feature_ident
    }

    pub fn get_config(&self) -> &[(String, String)] {
        &self.config
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features(pub HashMap<String, FeatureConfig>);

impl IntoIterator for Features {
    type Item = Feature;

    type IntoIter = std::vec::IntoIter<Feature>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.0.drain()
            .map(|(feature_ident, mut config)| Feature {
                feature_ident, config: config.0.drain().collect()
            })
            .collect_vec().into_iter()
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureConfig(pub HashMap<String, String>);

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(try_from = "&str")]
pub enum ProjectType {
    Executable,
    Library,
}

impl TryFrom<&str> for ProjectType {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "executable" => Ok(Self::Executable),
            "library" => Ok(Self::Library),
            _ => Err(SystemError::new(format!("'{value}' is not a valid project type!")).boxed())
        }
    }
}

impl Serialize for ProjectType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            ProjectType::Executable => "executable",
            ProjectType::Library => "library",
        }.serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfiguration {
    pub project: ProjectType,
    pub name: String,
    pub otr_version: Version,
    pub root_module: String,

    pub features: Features,
}

impl ProjectConfiguration {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_otr_version(&self) -> Version {
        self.otr_version
    }

    pub fn get_root_module(&self) -> &String {
        &self.root_module
    }

    pub fn get_features(&self) -> &Features {
        &self.features
    }

    pub fn features(self) -> Features {
        self.features
    }
}

pub fn get_project_config(config_path: &Path) -> otr_core::error::Result<ProjectConfiguration> {
    
    let config_string = fs::read_to_string(config_path).map_err(|err| SystemError::new(
        format!("Could not read project configuration file: {}", err)
    ).boxed())?;

    let config = toml::from_str(&config_string).map_err(|err| SystemError::new(
        format!("Could not parse project configuration file: {}", err)
    ).boxed())?;

    Ok(config)
}


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalConfiguration {
    pub roots: HashMap<String, PathBuf>,
}

impl GlobalConfiguration {

    pub fn new() -> Self {
        Self {
            roots: HashMap::new()
        }
    }

    pub fn try_resolve_root(&self, alias: impl AsRef<str>) -> Option<&Path> {
        self.roots.get(alias.as_ref()).map(PathBuf::as_ref)
    }
}

pub fn get_global_config(global_config_path: &Path) -> otr_core::error::Result<GlobalConfiguration> {
    
    let config_string = fs::read_to_string(global_config_path).map_err(|err| SystemError::new(
        format!("Could not read global configuration file: {}", err)
    ).boxed())?;

    let config = toml::from_str(&config_string).map_err(|err| SystemError::new(
        format!("Could not parse global configuration file: {}", err)
    ).boxed())?;

    Ok(config)
}