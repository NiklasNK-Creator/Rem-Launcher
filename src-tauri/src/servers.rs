//! Saved multiplayer servers and lightweight TCP reachability checks.
use serde::{Deserialize, Serialize};
use std::net::{ToSocketAddrs, TcpStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry { pub id: String, pub name: String, pub address: String, pub last_ping_ms: Option<u128> }

fn path(app: &AppHandle) -> Result<PathBuf, String> { Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("servers.json")) }
fn load(app: &AppHandle) -> Vec<ServerEntry> { std::fs::read_to_string(path(app).unwrap_or_default()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default() }
fn save(app: &AppHandle, v: &[ServerEntry]) -> Result<(), String> { let p=path(app)?; if let Some(d)=p.parent(){std::fs::create_dir_all(d).map_err(|e|e.to_string())?;} std::fs::write(p, serde_json::to_string_pretty(v).map_err(|e|e.to_string())?).map_err(|e|e.to_string()) }
fn valid_address(a: &str) -> bool { !a.is_empty() && !a.contains(['/', '\\', ' ', '\n', '\r']) && a.len() <= 255 }
fn valid_name(n: &str) -> bool { let n=n.trim(); !n.is_empty() && n.len() <= 80 && !n.contains(['\n', '\r']) }

#[tauri::command]
pub fn list_servers(app: AppHandle) -> Vec<ServerEntry> { load(&app) }

#[tauri::command]
pub fn add_server(app: AppHandle, name: String, address: String) -> Result<Vec<ServerEntry>, String> {
    if !valid_name(&name) { return Err("server name must be 1-80 characters".into()); }
    let address = address.trim().to_string();
    if !valid_address(&address) { return Err("invalid server address".into()); }
    let mut v = load(&app);
    let id = format!("server:{}", address.to_lowercase());
    v.retain(|s| s.id != id);
    v.push(ServerEntry { id, name: name.trim().to_string(), address, last_ping_ms: None });
    save(&app, &v)?;
    Ok(v)
}
#[tauri::command]
pub fn remove_server(app: AppHandle, id: String) -> Result<Vec<ServerEntry>, String> { let mut v=load(&app); v.retain(|s|s.id!=id); save(&app,&v)?; Ok(v) }
#[tauri::command]
pub fn ping_server(app: AppHandle, id: String) -> Result<ServerEntry, String> {
    let mut v=load(&app); let s=v.iter_mut().find(|s|s.id==id).ok_or("unknown server")?;
    let addr=if s.address.contains(':'){s.address.clone()}else{format!("{}:25565",s.address)};
    let socket=addr.to_socket_addrs().map_err(|e|e.to_string())?.next().ok_or("could not resolve server")?;
    let start=Instant::now(); TcpStream::connect_timeout(&socket,Duration::from_secs(3)).map_err(|e|e.to_string())?; s.last_ping_ms=Some(start.elapsed().as_millis()); let out=s.clone(); save(&app,&v)?; Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_server_names() {
        assert!(valid_name("Hypixel"));
        assert!(!valid_name(""));
        assert!(!valid_name("   "));
        assert!(!valid_name("a\nb"));
        assert!(!valid_name(&"x".repeat(81)));
    }
    #[test]
    fn validates_server_addresses() {
        assert!(valid_address("play.example.com"));
        assert!(valid_address("play.example.com:25565"));
        assert!(!valid_address(""));
        assert!(!valid_address("play example.com"));
        assert!(!valid_address("play/example.com:25565"));
    }
}
