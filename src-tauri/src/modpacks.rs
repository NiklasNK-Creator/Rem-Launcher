//! Modrinth .mrpack import. Spec: docs/providers/mrpack.md (sourced from
//! Modrinth help center). Security: path validation + domain whitelist.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Deserialize)]
struct MrpackIndex {
    #[serde(rename = "formatVersion")]
    format_version: u32,
    name: String,
    #[serde(rename = "versionId")]
    version_id: String,
    dependencies: std::collections::HashMap<String, String>,
    files: Vec<MrpackFile>,
}

#[derive(Debug, Deserialize)]
struct MrpackFile {
    path: String,
    hashes: std::collections::HashMap<String, String>,
    env: Option<MrpackEnv>,
    downloads: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MrpackEnv {
    client: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportedPack {
    pub instance: String,
    pub files: usize,
}

const ALLOWED_HOSTS: [&str; 4] = [
    "cdn.modrinth.com",
    "github.com",
    "raw.githubusercontent.com",
    "gitlab.com",
];

fn safe_rel(path: &str) -> Option<PathBuf> {
    if path.is_empty() || path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
        return None;
    }
    if path.len() > 1 && path.as_bytes()[1] == b':' {
        return None;
    }
    let p = PathBuf::from(path.replace('/', &std::path::MAIN_SEPARATOR.to_string()));
    if p.is_absolute() || p.components().count() == 0 {
        return None;
    }
    Some(p)
}

fn url_host_ok(url: &str) -> bool {
    let Some(after) = url.split("://").nth(1) else { return false };
    let host = after.split('/').next().unwrap_or("");
    ALLOWED_HOSTS.iter().any(|h| host == *h || host.ends_with(&format!(".{h}")))
}

#[tauri::command]
pub async fn import_mrpack(app: AppHandle, path: String) -> Result<ImportedPack, String> {
    let raw = std::fs::read(&path).map_err(|e| e.to_string())?;
    let cursor = std::io::Cursor::new(raw);
    let mut zip = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;
    let index_raw = {
        let mut f = zip.by_name("modrinth.index.json").map_err(|e| e.to_string())?;
        let mut buf = vec![];
        std::io::copy(&mut f, &mut buf).map_err(|e| e.to_string())?;
        buf
    };
    let index: MrpackIndex = serde_json::from_slice(&index_raw).map_err(|e| e.to_string())?;
    if index.format_version != 1 {
        return Err(format!("unsupported mrpack formatVersion {}", index.format_version));
    }
    let game_version = index.dependencies.get("minecraft").cloned().unwrap_or_default();
    if game_version.is_empty() {
        return Err("mrpack has no minecraft dependency".into());
    }
    let loader = if index.dependencies.contains_key("fabric-loader") {
        "fabric"
    } else if index.dependencies.contains_key("quilt-loader") {
        "quilt"
    } else if index.dependencies.contains_key("neoforge") {
        "neoforge"
    } else if index.dependencies.contains_key("forge") {
        "forge"
    } else {
        "vanilla"
    }
    .to_string();

    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    // Unique instance name from pack name.
    let mut stem: String = index.name.chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect();
    stem.truncate(48);
    if stem.is_empty() {
        stem = format!("pack-{}", &index.version_id[..8.min(index.version_id.len())]);
    }
    let inst_dir = base.join("instances").join(&stem);
    if inst_dir.exists() {
        return Err(format!("instance {stem} already exists"));
    }
    for sub in ["mods", "resourcepacks", "shaderpacks", "datapacks", "config"] {
        std::fs::create_dir_all(inst_dir.join(sub)).map_err(|e| e.to_string())?;
    }
    // Register instance in instances.json.
    let store = base.join("instances.json");
    let mut list: Vec<serde_json::Value> =
        std::fs::read_to_string(&store).ok().and_then(|r| serde_json::from_str(&r).ok()).unwrap_or_default();
    list.push(serde_json::json!({ "name": stem, "game_version": game_version, "loader": loader }));
    std::fs::write(&store, serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let client = reqwest::Client::builder().user_agent("rem-launcher/0.1.0").build().map_err(|e| e.to_string())?;
    let mut count = 0;
    for file in &index.files {
        if file.env.as_ref().and_then(|e| e.client.as_deref()) == Some("unsupported") {
            continue;
        }
        let Some(rel) = safe_rel(&file.path) else { continue };
        let url = file.downloads.iter().find(|u| url_host_ok(u)).cloned().unwrap_or_default();
        if url.is_empty() {
            continue;
        }
        let bytes = client.get(&url).send().await.map_err(|e| e.to_string())?.bytes().await.map_err(|e| e.to_string())?;
        // Verify sha512 (or sha1 fallback) when present.
        if let Some(expect) = file.hashes.get("sha512") {
            use sha2::Digest;
            let mut h = sha2::Sha512::new();
            h.update(&bytes);
            let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
            if got != expect.to_lowercase() {
                continue;
            }
        } else if let Some(expect) = file.hashes.get("sha1") {
            use sha1::Digest;
            let mut h = sha1::Sha1::new();
            h.update(&bytes);
            let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
            if got != expect.to_lowercase() {
                continue;
            }
        }
        let dest = inst_dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        // Ensure dest stays inside instance dir.
        if !dest.starts_with(&inst_dir) {
            continue;
        }
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
        count += 1;
    }
    // Extract overrides/ + client-overrides/ (skip server-overrides).
    let names: Vec<String> = (0..zip.len()).filter_map(|i| zip.by_index(i).ok().map(|e| e.name().to_string())).collect();
    for prefix in ["overrides/", "client-overrides/"] {
        for name in names.iter().filter(|n| n.starts_with(prefix) && !n.ends_with('/')) {
            let rel = &name[prefix.len()..];
            let Some(safe) = safe_rel(rel) else { continue };
            let mut entry = zip.by_name(name).map_err(|e| e.to_string())?;
            let dest = inst_dir.join(&safe);
            if !dest.starts_with(&inst_dir) {
                continue;
            }
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(ImportedPack { instance: stem, files: count })
}
