use std::collections::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::error::system_error::SystemError;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(try_from = "&str")]
pub struct Version {
    major: usize,
    minor: usize,
    patch: usize,
}

impl Version {
    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self { major, minor, patch }
    }
}

impl TryFrom<&str> for Version {
    type Error = Box<dyn crate::error::Error>;

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
    feature_ident: String,
    config: Vec<(String, String)>
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
pub struct Features(HashMap<String, FeatureConfig>);

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
pub struct FeatureConfig(HashMap<String, String>);


#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfiguration {
    name: String,
    otr_version: Version,
    root_module: String,

    features: Features,
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
}