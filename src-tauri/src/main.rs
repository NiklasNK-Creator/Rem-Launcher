// Rem Launcher shell. Business logic lives in crates/* — keep handlers thin.
// Read docs/architecture.md before changing this system.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use launcher_core::models::{ContentType, SearchQuery};
use launcher_core::providers::modrinth::ModrinthProvider;
use launcher_core::providers::ContentProvider;
use std::sync::Arc;
use tauri::State;
mod accounts;
mod instances;
mod launch;

struct AppState {
    modrinth: Arc<ModrinthProvider>,
}

#[tauri::command]
async fn search_mods(
    state: State<'_, AppState>,
    text: String,
    game_version: Option<String>,
    loader: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<launcher_core::models::Project>, String> {
    let q = SearchQuery {
        text,
        content_type: Some(ContentType::Mod),
        game_versions: game_version.into_iter().collect(),
        loaders: loader.into_iter().collect(),
        limit: limit.unwrap_or(20),
        offset: 0,
    };
    state.modrinth.search(&q).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn project_versions(
    state: State<'_, AppState>,
    project_id: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
) -> Result<Vec<launcher_core::models::ProjectVersion>, String> {
    state
        .modrinth
        .versions(&project_id, &game_versions, &loaders)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn project_info(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<launcher_core::models::Project, String> {
    state.modrinth.project(&project_id).await.map_err(|e| e.to_string())
}

fn main() {
    let state = AppState {
        modrinth: Arc::new(ModrinthProvider::new()),
    };
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![search_mods, project_versions, project_info, accounts::list_accounts, accounts::add_account, accounts::remove_account, launch::prepare_client, launch::detect_java, launch::launch_game, launch::read_log_tail, launch::list_versions, instances::list_instances, instances::create_instance, instances::install_mod, instances::list_instance_mods, instances::remove_mod, instances::locked_mods])
        .run(tauri::generate_context!())
        .expect("failed to run Rem Launcher");
}
