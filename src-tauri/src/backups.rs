//! Instance world backup and restore system.
//! Read docs/architecture.md and docs/security.md before changing.
use crate::instances::{data_root, valid_instance_name};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub file_name: String,
    pub size_bytes: u64,
    pub created_at: String,
}

fn valid_backup_file_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        && name.ends_with(".zip")
}

fn backups_dir(app: &AppHandle, instance: &str) -> Result<PathBuf, String> {
    if !valid_instance_name(instance) {
        return Err("invalid instance name".into());
    }
    let p = data_root(app)?.join(instance.trim()).join("backups");
    std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

fn saves_dir(app: &AppHandle, instance: &str) -> Result<PathBuf, String> {
    if !valid_instance_name(instance) {
        return Err("invalid instance name".into());
    }
    let p = data_root(app)?.join(instance.trim()).join("saves");
    std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

#[tauri::command]
pub fn list_backups(app: AppHandle, instance: String) -> Result<Vec<BackupEntry>, String> {
    let dir = backups_dir(&app, &instance)?;
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".zip") {
                        let meta = e.metadata().ok();
                        let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                        let created_at = meta
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs().to_string())
                            .unwrap_or_else(|| "0".into());
                        out.push(BackupEntry {
                            file_name: name.to_string(),
                            size_bytes,
                            created_at,
                        });
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    base: &Path,
    current: &Path,
    opts: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    if !current.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(current).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let rel = path
            .strip_prefix(base)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            zip.add_directory(&rel, opts).map_err(|e| e.to_string())?;
            add_dir_to_zip(zip, base, &path, opts)?;
        } else if path.is_file() {
            zip.start_file(&rel, opts).map_err(|e| e.to_string())?;
            let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_backup(
    app: AppHandle,
    instance: String,
    backup_name: Option<String>,
) -> Result<BackupEntry, String> {
    let b_dir = backups_dir(&app, &instance)?;
    let s_dir = saves_dir(&app, &instance)?;

    let now_sec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let file_name = match backup_name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        Some(custom) => {
            let base = custom.trim_end_matches(".zip");
            let clean: String = base
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                .collect();
            format!("{clean}-{now_sec}.zip")
        }
        None => format!("worlds-{now_sec}.zip"),
    };

    let target_path = b_dir.join(&file_name);
    let file = std::fs::File::create(&target_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();

    add_dir_to_zip(&mut zip, &s_dir, &s_dir, opts)?;
    zip.finish().map_err(|e| e.to_string())?;

    let size_bytes = std::fs::metadata(&target_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(BackupEntry {
        file_name,
        size_bytes,
        created_at: now_sec.to_string(),
    })
}

#[tauri::command]
pub fn restore_backup(app: AppHandle, instance: String, file_name: String) -> Result<(), String> {
    if !valid_backup_file_name(&file_name) {
        return Err("invalid backup file name".into());
    }
    let b_dir = backups_dir(&app, &instance)?;
    let s_dir = saves_dir(&app, &instance)?;
    let zip_path = b_dir.join(&file_name);
    if !zip_path.exists() {
        return Err("backup file does not exist".into());
    }

    let file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        // Zip-slip guard: reject paths with .. or absolute indicators
        if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
            continue;
        }
        if name.len() > 1 && name.as_bytes()[1] == b':' {
            continue;
        }

        let out_path = s_dir.join(name.replace('/', &std::path::MAIN_SEPARATOR.to_string()));
        if !out_path.starts_with(&s_dir) {
            continue;
        }

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn delete_backup(
    app: AppHandle,
    instance: String,
    file_name: String,
) -> Result<Vec<BackupEntry>, String> {
    if !valid_backup_file_name(&file_name) {
        return Err("invalid backup file name".into());
    }
    let b_dir = backups_dir(&app, &instance)?;
    let target = b_dir.join(&file_name);
    if target.exists() {
        std::fs::remove_file(target).map_err(|e| e.to_string())?;
    }
    list_backups(app, instance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_backup_names() {
        assert!(valid_backup_file_name("worlds-123.zip"));
        assert!(valid_backup_file_name("my_world.zip"));
        assert!(!valid_backup_file_name(""));
        assert!(!valid_backup_file_name("worlds.tar"));
        assert!(!valid_backup_file_name("../evil.zip"));
        assert!(!valid_backup_file_name(".hidden.zip"));
        assert!(!valid_backup_file_name("a/b.zip"));
        assert!(!valid_backup_file_name("a\\b.zip"));
        assert!(!valid_backup_file_name("world:name.zip"));
    }
}
