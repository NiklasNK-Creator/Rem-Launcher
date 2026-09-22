// Account store: multiple Microsoft + offline ("cracked") profiles.
// Read docs/auth.md before changing. Microsoft tokens live in the OS
// keychain (Credential Manager / Secret Service / Keychain) — accounts.json
// NEVER contains secrets.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub kind: AccountKind,
    pub mc_name: String,
    pub uuid: String,
    /// Transient only: carried in on add, stored to keychain, never persisted.
    pub ms_refresh: Option<serde_json::Value>,
    pub last_used: Option<String>,
    #[serde(default)]
    pub skin_url: Option<String>,
    #[serde(default)]
    pub cape_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Microsoft,
    Offline,
}

const KEYCHAIN_SERVICE: &str = "dev.remlauncher.app";

fn token_key(id: &str) -> String {
    format!("ms-token:{id}")
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
    let scrubbed: Vec<Account> = accounts
        .iter()
        .map(|a| Account { ms_refresh: None, ..a.clone() })
        .collect();
    let raw = serde_json::to_string_pretty(&scrubbed).map_err(|e| e.to_string())?;
    std::fs::write(path, raw).map_err(|e| e.to_string())
}

fn epoch_now() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs().to_string(),
        Err(_) => "0".into(),
    }
}

#[tauri::command]
pub fn list_accounts(app: AppHandle) -> Vec<Account> {
    load_all(&app)
}

#[tauri::command]
pub fn get_account_token(id: String) -> Option<String> {
    keyring::Entry::new(KEYCHAIN_SERVICE, &token_key(&id))
        .ok()
        .and_then(|e| e.get_password().ok())
}

#[tauri::command]
pub fn touch_account(app: AppHandle, id: String) -> Vec<Account> {
    let mut all = load_all(&app);
    for a in &mut all {
        if a.id == id {
            a.last_used = Some(epoch_now());
        }
    }
    let _ = save_all(&app, &all);
    all
}

#[tauri::command]
pub fn add_account(app: AppHandle, account: Account) -> Result<Vec<Account>, String> {
    if let Some(secret) = &account.ms_refresh {
        let raw = serde_json::to_string(secret).map_err(|e| e.to_string())?;
        let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &token_key(&account.id))
            .map_err(|e| e.to_string())?;
        entry.set_password(&raw).map_err(|e| e.to_string())?;
    }
    let mut all = load_all(&app);
    all.retain(|a| a.id != account.id);
    all.push(account);
    save_all(&app, &all)?;
    Ok(all)
}

#[tauri::command]
pub fn remove_account(app: AppHandle, id: String) -> Result<Vec<Account>, String> {
    if let Ok(entry) = keyring::Entry::new(KEYCHAIN_SERVICE, &token_key(&id)) {
        let _ = entry.delete_credential();
    }
    let mut all = load_all(&app);
    all.retain(|a| a.id != id);
    save_all(&app, &all)?;
    Ok(all)
}
