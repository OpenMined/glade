use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DATABASES_YAML: &str = include_str!("databases.yaml");

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(flatten)]
    pub databases: HashMap<String, DatabaseVersions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseVersions {
    #[serde(flatten)]
    pub versions: HashMap<String, DatabaseFiles>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseFiles {
    pub vcf: String,
    pub tbi: String,
    pub md5: String,
}

pub fn load_config() -> crate::Result<HashMap<String, HashMap<String, DatabaseFiles>>> {
    serde_yaml::from_str(DATABASES_YAML).map_err(Into::into)
}
