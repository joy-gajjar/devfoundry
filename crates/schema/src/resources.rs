use serde::{Deserialize, Serialize};

use crate::{ProjectId, Revision};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceManifest {
    pub id: String,
    pub version: String,
    pub source: String,
    pub manifest_hash: String,
    pub files: Vec<ResourceFile>,
    pub required_capabilities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceFile {
    pub target: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstalledResource {
    pub id: String,
    pub project_id: ProjectId,
    pub version: String,
    pub revision: Revision,
    pub manifest_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstalledResourceFile {
    pub resource_id: String,
    pub project_id: ProjectId,
    pub target: String,
    pub expected_hash: String,
    pub installed_hash: String,
    pub size: u64,
}
