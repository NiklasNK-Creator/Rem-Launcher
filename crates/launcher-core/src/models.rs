//! Provider-neutral content models.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Mod,
    ResourcePack,
    DataPack,
    Shader,
    Modpack,
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub content_type: Option<ContentType>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub limit: u32,
    pub offset: u32,
    /// relevance (default), downloads, follows, newest, updated
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub provider: String,
    pub id: String,
    pub slug: Option<String>,
    pub title: String,
    pub description: String,
    pub content_type: ContentType,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub icon_url: Option<String>,
    pub downloads: u64,
    #[serde(default)]
    pub follows: u64,
    #[serde(default)]
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    pub name: String,
    pub url: String,
    pub sha512: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDependency {
    pub project_id: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<VersionFile>,
    #[serde(default)]
    pub dependencies: Vec<VersionDependency>,
}
