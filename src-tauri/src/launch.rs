//! Launch preparation: resolve a Mojang version, download its client jar
//! (sha1-verified), and detect a Java runtime.
//! Read docs/architecture.md before changing this system.
use launcher_install::{download_verified, fetch_version_manifest, InstallError};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize)]
pub struct PreparedClient {
    pub version: String,
    pub jar_path: String,
}

#[derive(Debug, Serialize)]
pub struct JavaInfo {
    pub path: String,
    pub version_line: String,
}

#[tauri::command]
pub async fn prepare_client(app: AppHandle, version: String) -> Result<PreparedClient, String> {
    let client = reqwest::Client::builder()
        .user_agent("rem-launcher/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;
    let entries = fetch_version_manifest(&client).await.map_err(stringify)?;
    let entry = entries
        .iter()
        .find(|e| e.id == version)
        .ok_or_else(|| format!("unknown version {version}"))?;
    // Per-version package describes client jar + sha1.
    #[derive(serde::Deserialize)]
    struct Pkg {
        downloads: std::collections::HashMap<String, Artifact>,
    }
    #[derive(serde::Deserialize)]
    struct Artifact {
        sha1: String,
        url: String,
    }
    let pkg: Pkg = client
        .get(&entry.url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let artifact = pkg
        .downloads
        .get("client")
        .ok_or_else(|| "version package has no client download".to_string())?;
    let bytes = download_verified(&client, &artifact.url, &artifact.sha1)
        .await
        .map_err(stringify)?;
    let dir: PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("versions")
        .join(&version);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let jar = dir.join("client.jar");
    std::fs::write(&jar, bytes).map_err(|e| e.to_string())?;
    Ok(PreparedClient {
        version,
        jar_path: jar.to_string_lossy().into(),
    })
}

#[tauri::command]
pub fn detect_java() -> Result<JavaInfo, String> {
    for candidate in ["java", "javaw"] {
        if let Ok(out) = std::process::Command::new(candidate)
            .arg("-version")
            .output()
        {
            let line = String::from_utf8_lossy(&out.stderr)
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            if !line.is_empty() {
                return Ok(JavaInfo {
                    path: candidate.into(),
                    version_line: line,
                });
            }
        }
    }
    Err("no Java runtime found on PATH".into())
}

fn stringify(e: InstallError) -> String {
    e.to_string()
}
