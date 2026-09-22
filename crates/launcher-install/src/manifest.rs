//! Mojang version manifests + hash-verified downloads.
use serde::{Deserialize, Serialize};
use sha1::Digest;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("http: {0}")]
    Http(String),
    #[error("hash mismatch for {0}")]
    HashMismatch(String),
    #[error("io: {0}")]
    Io(String),
}

pub type Result<T, E = InstallError> = std::result::Result<T, E>;

pub const VERSION_MANIFEST_V2: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionEntry {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub releaseTime: String,
    pub sha1: String,
}

/// Fetch the version list (network). Returns raw entries.
pub async fn fetch_version_manifest(client: &reqwest::Client) -> Result<Vec<VersionEntry>> {
    #[derive(Deserialize)]
    struct Manifest {
        versions: Vec<VersionEntry>,
    }
    let m: Manifest = client
        .get(VERSION_MANIFEST_V2)
        .send()
        .await
        .map_err(|e| InstallError::Http(e.to_string()))?
        .json()
        .await
        .map_err(|e| InstallError::Http(e.to_string()))?;
    Ok(m.versions)
}

/// Download bytes and verify sha1 hex.
pub async fn download_verified(
    client: &reqwest::Client,
    url: &str,
    expect_sha1: &str,
) -> Result<Vec<u8>> {
    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|e| InstallError::Http(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| InstallError::Http(e.to_string()))?;
    let mut h = sha1::Sha1::new();
    h.update(&bytes);
    let got = hex_of(h.finalize());
    if got != expect_sha1.to_lowercase() {
        return Err(InstallError::HashMismatch(url.into()));
    }
    Ok(bytes.to_vec())
}

fn hex_of(digest: sha1::digest::Output<sha1::Sha1>) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hex_known() {
        let mut h = sha1::Sha1::new();
        h.update(b"abc");
        assert_eq!(hex_of(h.finalize()), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }
}
