//! Instance store + mod installation.
//! Read docs/architecture.md before changing this system.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub game_version: String,
    pub loader: String,
}

fn store_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("instances.json"))
}

fn data_root(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("instances"))
}

fn load_all(app: &AppHandle) -> Vec<Instance> {
    let path = match store_path(app) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_all(app: &AppHandle, list: &[Instance]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    std::fs::write(store_path(app)?, raw).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_instances(app: AppHandle) -> Vec<Instance> {
    load_all(&app)
}

#[tauri::command]
pub fn create_instance(
    app: AppHandle,
    name: String,
    game_version: String,
    loader: String,
) -> Result<Vec<Instance>, String> {
    let name = name.trim();
    if name.is_empty() || name.len() > 64 {
        return Err("instance name must be 1-64 chars".into());
    }
    if name.contains(['/', '\\', '.', ':']) {
        return Err("instance name contains invalid characters".into());
    }
    let mut all = load_all(&app);
    if all.iter().any(|i| i.name == name) {
        return Err("instance already exists".into());
    }
    std::fs::create_dir_all(data_root(&app)?.join(name).join("mods"))
        .map_err(|e| e.to_string())?;
    all.push(Instance { name: name.into(), game_version, loader });
    save_all(&app, &all)?;
    Ok(all)
}

#[tauri::command]
pub async fn install_mod(
    app: AppHandle,
    instance: String,
    file_name: String,
    url: String,
    sha512: Option<String>,
) -> Result<String, String> {
    // Confine to <data>/instances/<instance>/mods/<file_name>, single component.
    if file_name.contains(['/', '\\']) || file_name.starts_with('.') || file_name.is_empty() {
        return Err("invalid file name".into());
    }
    let all = load_all(&app);
    if !all.iter().any(|i| i.name == instance) {
        return Err("unknown instance".into());
    }
    let dest = data_root(&app)?.join(&instance).join("mods").join(&file_name);
    let bytes = reqwest::get(&url).await.map_err(|e| e.to_string())?
        .bytes().await.map_err(|e| e.to_string())?;
    if let Some(expect) = sha512 {
        use sha2::Digest;
        let mut h = sha2::Sha512::new();
        h.update(&bytes);
        let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if got != expect.to_lowercase() {
            return Err(format!("sha512 mismatch for {file_name}"));
        }
    }
    std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().into())
}

#[tauri::command]
pub fn list_instance_mods(app: AppHandle, instance: String) -> Result<Vec<String>, String> {
    let dir = data_root(&app)?.join(&instance).join("mods");
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if let Some(n) = e.file_name().to_str() {
                out.push(n.to_string());
            }
        }
    }
    out.sort();
    Ok(out)
}

#[tauri::command]
pub fn remove_mod(app: AppHandle, instance: String, file_name: String) -> Result<Vec<String>, String> {
    if file_name.contains(['/', '\\']) || file_name.starts_with('.') || file_name.is_empty() {
        return Err("invalid file name".into());
    }
    let target = data_root(&app)?.join(&instance).join("mods").join(&file_name);
    if target.exists() {
        std::fs::remove_file(&target).map_err(|e| e.to_string())?;
    }
    list_instance_mods(app, instance)
}
