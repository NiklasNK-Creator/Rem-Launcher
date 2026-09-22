// Account store: multiple Microsoft + offline ("cracked") profiles.
// Read docs/auth.md before changing. Secrets note: Microsoft refresh tokens
// arrive from prismarine-auth (frontend) and are persisted here as JSON for
// now; OS-keychain encryption is tracked as a follow-up in docs/auth.md.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub kind: AccountKind,
    pub mc_name: String,
    pub uuid: String,
    /// Microsoft refresh material from prismarine-auth (opaque JSON). None for offline.
    pub ms_refresh: Option<serde_json::Value>,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Microsoft,
    Offline,
}

fn store_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("accounts.json"))
}

fn load_all(app: &AppHandle) -> Vec<Account> {
    let path = match store_path(app) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_all(app: &AppHandle, accounts: &[Account]) -> Result<(), String> {
    let path = store_path(app)?;
    let raw = serde_json::to_string_pretty(accounts).map_err(|e| e.to_string())?;
    std::fs::write(path, raw).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_accounts(app: AppHandle) -> Vec<Account> {
    load_all(&app)
}

#[tauri::command]
pub fn add_account(app: AppHandle, account: Account) -> Result<Vec<Account>, String> {
    let mut all = load_all(&app);
    all.retain(|a| a.id != account.id);
    all.push(account);
    save_all(&app, &all)?;
    Ok(all)
}

#[tauri::command]
pub fn remove_account(app: AppHandle, id: String) -> Result<Vec<Account>, String> {
    let mut all = load_all(&app);
    all.retain(|a| a.id != id);
    save_all(&app, &all)?;
    Ok(all)
}
