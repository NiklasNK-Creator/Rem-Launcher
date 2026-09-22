//! Instance store + mod installation with lockfile.
//! Read docs/architecture.md before changing this system.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    #[serde(default)]
    pub jvm_args: Option<String>,
    #[serde(default)]
    pub max_memory_mb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub project_id: String,
    pub version_number: String,
    pub file_name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha512: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
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

/// Shared name guard: instance names become directory names — reject
/// traversal, hidden, absolute, and Windows-special characters in one place.
/// Read docs/security.md before loosening this.
pub fn valid_instance_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty()
        && n.len() <= 64
        && !n.starts_with('.')
        && !n.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        && n != ".."
        && n != "."
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

/// Delete an instance: remove store entry + its on-disk dir.
#[tauri::command]
pub fn delete_instance(app: AppHandle, name: String) -> Result<Vec<Instance>, String> {
    if !valid_instance_name(&name) {
        return Err("invalid instance name".into());
    }
    let trimmed = name.trim();
    let mut all = load_all(&app);
    all.retain(|i| i.name != trimmed);
    save_all(&app, &all)?;
    let dir = data_root(&app)?.join(trimmed);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(all)
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Clone/duplicate an existing instance with all its installed mods, configs, and settings.
#[tauri::command]
pub fn clone_instance(app: AppHandle, source_name: String, new_name: String) -> Result<Vec<Instance>, String> {
    if !valid_instance_name(&source_name) || !valid_instance_name(&new_name) {
        return Err("invalid instance name".into());
    }
    let src_trim = source_name.trim();
    let new_trim = new_name.trim();
    if src_trim == new_trim {
        return Err("new name must be different from source name".into());
    }
    let mut all = load_all(&app);
    let src = all.iter().find(|i| i.name == src_trim).cloned().ok_or("source instance does not exist")?;
    if all.iter().any(|i| i.name == new_trim) {
        return Err("instance with this name already exists".into());
    }
    let src_dir = data_root(&app)?.join(src_trim);
    let new_dir = data_root(&app)?.join(new_trim);
    if src_dir.exists() {
        copy_dir_all(&src_dir, &new_dir).map_err(|e| format!("failed to copy instance files: {e}"))?;
    } else {
        std::fs::create_dir_all(&new_dir).map_err(|e| e.to_string())?;
    }
    let cloned = Instance {
        name: new_trim.into(),
        game_version: src.game_version,
        loader: src.loader,
        jvm_args: src.jvm_args,
        max_memory_mb: src.max_memory_mb,
    };
    all.push(cloned);
    save_all(&app, &all)?;
    Ok(all)
}

fn lockfile(app: &AppHandle, instance: &str) -> Result<PathBuf, String> {
    Ok(data_root(app)?.join(instance).join("installed.json"))
}

fn load_locked(app: &AppHandle, instance: &str) -> Vec<InstalledMod> {
    let path = match lockfile(app, instance) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_locked(app: &AppHandle, instance: &str, mods: &[InstalledMod]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(mods).map_err(|e| e.to_string())?;
    std::fs::write(lockfile(app, instance)?, raw).map_err(|e| e.to_string())
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
    if !valid_instance_name(&name) {
        return Err("instance name must be 1-64 chars without / \\ : * ? \" < > | or leading dots".into());
    }
    let name = name.trim();
    let mut all = load_all(&app);
    if all.iter().any(|i| i.name == name) {
        return Err("instance already exists".into());
    }
    for sub in ["mods", "resourcepacks", "shaderpacks", "datapacks"] {
        std::fs::create_dir_all(data_root(&app)?.join(name).join(sub)).map_err(|e| e.to_string())?;
    }
    all.push(Instance {
        name: name.into(),
        game_version,
        loader,
        jvm_args: None,
        max_memory_mb: None,
    });
    save_all(&app, &all)?;
    Ok(all)
}

#[tauri::command]
pub fn configure_instance(
    app: AppHandle,
    name: String,
    max_memory_mb: Option<u32>,
    jvm_args: Option<String>,
) -> Result<Vec<Instance>, String> {
    if !valid_instance_name(&name) {
        return Err("invalid instance name".into());
    }
    let mut all = load_all(&app);
    let trimmed = name.trim();
    if let Some(inst) = all.iter_mut().find(|i| i.name == trimmed) {
        inst.max_memory_mb = max_memory_mb;
        inst.jvm_args = jvm_args.filter(|s| !s.trim().is_empty());
        save_all(&app, &all)?;
        Ok(all)
    } else {
        Err("unknown instance".into())
    }
}

#[tauri::command]
pub fn open_instance_folder(app: AppHandle, name: String) -> Result<(), String> {
    if !valid_instance_name(&name) {
        return Err("invalid instance name".into());
    }
    let dir = data_root(&app)?.join(name.trim());
    if !dir.exists() {
        return Err("instance folder does not exist".into());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn valid_file_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains(['/', '\\'])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guards_traversal_and_specials() {
        assert!(valid_instance_name("survival"));
        assert!(!valid_instance_name(""));
        assert!(!valid_instance_name("../evil"));
        assert!(!valid_instance_name("a/b"));
        assert!(!valid_instance_name(".."));
        assert!(!valid_instance_name(".hidden"));
        assert!(!valid_instance_name("con:drive"));
        assert!(!valid_instance_name("a*b"));
    }
}

#[tauri::command]
pub async fn install_mod(
    app: AppHandle,
    instance: String,
    file_name: String,
    url: String,
    sha512: Option<String>,
    project_id: Option<String>,
    version_number: Option<String>,
    content_type: Option<String>,
) -> Result<String, String> {
    if !valid_file_name(&file_name) {
        return Err("invalid file name".into());
    }
    let subdir = match content_type.as_deref() {
        Some("resourcepack") => "resourcepacks",
        Some("shader") => "shaderpacks",
        Some("datapack") => "datapacks",
        _ => "mods",
    };
    let all = load_all(&app);
    if !all.iter().any(|i| i.name == instance) {
        return Err("unknown instance".into());
    }
    let dest = data_root(&app)?.join(&instance).join(subdir).join(&file_name);
    let bytes = reqwest::get(&url).await.map_err(|e| e.to_string())?
        .bytes().await.map_err(|e| e.to_string())?;
    if let Some(expect) = sha512.as_deref() {
        use sha2::Digest;
        let mut h = sha2::Sha512::new();
        h.update(&bytes);
        let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if got != expect.to_lowercase() {
            return Err(format!("sha512 mismatch for {file_name}"));
        }
    }
    std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
    if let (Some(pid), Some(ver)) = (project_id, version_number) {
        let mut locked = load_locked(&app, &instance);
        locked.retain(|m| m.file_name != file_name && m.project_id != pid);
        locked.push(InstalledMod {
            project_id: pid,
            version_number: ver,
            file_name: file_name.clone(),
            url: Some(url),
            sha512,
            content_type,
        });
        save_locked(&app, &instance, &locked)?;
    }
    Ok(dest.to_string_lossy().into())
}

fn content_dir(content_type: Option<&str>) -> &'static str {
    match content_type {
        Some("resourcepack") => "resourcepacks",
        Some("shader") => "shaderpacks",
        Some("datapack") => "datapacks",
        _ => "mods",
    }
}

#[tauri::command]
pub fn list_instance_mods(app: AppHandle, instance: String) -> Result<Vec<String>, String> {
    let root = data_root(&app)?.join(&instance);
    let locked = load_locked(&app, &instance);
    let mut out = vec![];
    for subdir in ["mods", "resourcepacks", "shaderpacks", "datapacks"] {
        if let Ok(rd) = std::fs::read_dir(root.join(subdir)) {
            for e in rd.flatten() {
                if let Some(n) = e.file_name().to_str() {
                    out.push(format!("{subdir}/{n}"));
                }
            }
        }
    }
    // Preserve legacy UI behavior while exposing all content directories.
    if out.is_empty() && locked.is_empty() {
        return Ok(vec![]);
    }
    out.sort();
    Ok(out)
}

#[tauri::command]
pub fn remove_content_item(app: AppHandle, instance: String, relative_path: String) -> Result<Vec<String>, String> {
    let (subdir, leaf) = relative_path.split_once('/').ok_or("content path must include a directory")?;
    if !["mods", "resourcepacks", "shaderpacks", "datapacks"].contains(&subdir) || !valid_file_name(leaf) {
        return Err("invalid content file name".into());
    }
    let target = data_root(&app)?.join(&instance).join(subdir).join(leaf);
    if target.exists() {
        std::fs::remove_file(&target).map_err(|e| e.to_string())?;
    }
    let mut locked = load_locked(&app, &instance);
    locked.retain(|m| !(m.file_name == leaf && content_dir(m.content_type.as_deref()) == subdir));
    let _ = save_locked(&app, &instance, &locked);
    list_instance_mods(app, instance)
}

#[tauri::command]
pub fn remove_mod(app: AppHandle, instance: String, file_name: String) -> Result<Vec<String>, String> {
    let path = if file_name.contains('/') {
        file_name
    } else {
        format!("mods/{file_name}")
    };
    remove_content_item(app, instance, path)
}

#[tauri::command]
pub fn locked_mods(app: AppHandle, instance: String) -> Vec<InstalledMod> {
    load_locked(&app, &instance)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReport {
    pub missing_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ok_tracked: usize,
}

/// Offline check: compare lockfile against files actually on disk.
#[tauri::command]
pub fn verify_instance(app: AppHandle, instance: String) -> Result<VerifyReport, String> {
    let locked = load_locked(&app, &instance);
    let on_disk = list_instance_mods(app.clone(), instance.clone())?;
    let mut missing = vec![];
    let mut ok = 0;
    let mut tracked_paths = std::collections::HashSet::new();
    for m in &locked {
        let full_rel = format!("{}/{}", content_dir(m.content_type.as_deref()), m.file_name);
        if on_disk.contains(&full_rel) || on_disk.contains(&m.file_name) {
            ok += 1;
            tracked_paths.insert(full_rel);
            tracked_paths.insert(m.file_name.clone());
        } else {
            missing.push(m.file_name.clone());
        }
    }
    let untracked: Vec<String> = on_disk
        .into_iter()
        .filter(|f| !tracked_paths.contains(f))
        .collect();
    Ok(VerifyReport { missing_files: missing, untracked_files: untracked, ok_tracked: ok })
}

/// Repair: drop lockfile entries whose files are gone (untracked files stay).
#[tauri::command]
pub fn repair_instance(app: AppHandle, instance: String) -> Result<VerifyReport, String> {
    let report = verify_instance(app.clone(), instance.clone())?;
    if !report.missing_files.is_empty() {
        let mut locked = load_locked(&app, &instance);
        locked.retain(|m| !report.missing_files.contains(&m.file_name));
        save_locked(&app, &instance, &locked)?;
    }
    verify_instance(app, instance)
}
