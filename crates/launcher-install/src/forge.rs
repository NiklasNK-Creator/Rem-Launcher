//! Forge/NeoForge installer profile parsing.
//! Processor execution remains intentionally separate because installer
//! processors are versioned Java tools with provider-specific classpaths.
use serde::Deserialize;
use std::io::Read;
use zip::ZipArchive;

#[derive(Debug, Clone, Deserialize)]
pub struct InstallerProfile {
    pub spec: Option<u32>,
    pub version: Option<String>,
    pub path: Option<String>,
    #[serde(default)]
    pub processors: Vec<InstallerProcessor>,
    #[serde(default)]
    pub libraries: Vec<InstallerLibrary>,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallerProcessor {
    pub sides: Option<Vec<String>>,
    pub jar: String,
    #[serde(default)]
    pub classpath: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallerLibrary {
    pub name: String,
    pub downloads: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("installer is not a valid ZIP: {0}")]
    Zip(String),
    #[error("installer has no install_profile.json")]
    MissingProfile,
    #[error("invalid install_profile.json: {0}")]
    Json(String),
}

pub fn parse_install_profile(bytes: &[u8]) -> Result<InstallerProfile, ProfileError> {
    let mut zip = ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| ProfileError::Zip(e.to_string()))?;
    let mut raw = String::new();
    let mut file = zip
        .by_name("install_profile.json")
        .map_err(|_| ProfileError::MissingProfile)?;
    file.read_to_string(&mut raw)
        .map_err(|e| ProfileError::Json(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| ProfileError::Json(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_zip() {
        assert!(matches!(parse_install_profile(b"not zip"), Err(ProfileError::Zip(_))));
    }
}
