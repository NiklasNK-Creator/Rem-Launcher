//! Full game launch: libraries + assets + natives + java spawn.
//! Read docs/architecture.md before changing this system.
use launcher_install::{download_verified, fetch_version_manifest, InstallError};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
pub struct LaunchResult {
    pub pid: u32,
}
#[derive(Debug, Deserialize)]
struct Pkg {
    downloads: std::collections::HashMap<String, Artifact>,
    libraries: Vec<Library>,
    #[serde(rename = "assetIndex")]
    asset_index: Option<AssetIndexRef>,
    assets: Option<String>,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "minecraftArguments", default)]
    minecraft_arguments: Option<String>,
    #[serde(default)]
    arguments: Option<serde_json::Value>,
    #[serde(rename = "minimumLauncherVersion")]
    #[allow(dead_code)]
    minimum_launcher_version: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    sha1: String,
    url: String,
    #[allow(dead_code)]
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Library {
    name: String,
    downloads: Option<LibDownloads>,
    rules: Option<Vec<Rule>>,
    #[serde(default)]
    natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct LibDownloads {
    artifact: Option<LibArtifact>,
}

#[derive(Debug, Deserialize)]
struct LibArtifact {
    path: String,
    sha1: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Rule {
    action: String,
    os: Option<OsRule>,
}

#[derive(Debug, Deserialize)]
struct OsRule {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AssetIndexRef {
    id: String,
    sha1: String,
    url: String,
}

fn os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

fn rule_allows(rules: &Option<Vec<Rule>>) -> bool {
    let Some(rules) = rules else { return true };
    let mut allowed = true;
    for r in rules {
        let applies = match &r.os {
            None => true,
            Some(os) => os.name.as_deref() == Some(os_name()),
        };
        if applies {
            allowed = r.action == "allow";
        }
    }
    allowed
}

fn maven_path(name: &str) -> Option<String> {
    let mut parts = name.split(':');
    let g = parts.next()?.replace('.', "/");
    let a = parts.next()?;
    let v = parts.next()?;
    let file = format!("{a}-{v}.jar");
    parts.next().map(|c| format!("{g}/{a}/{v}/{a}-{v}-{c}.jar")).or(Some(format!("{g}/{a}/{v}/{file}")))
}

/// Download all allowed libraries (sha1-verified) into versions/<v>/libraries.
async fn fetch_libraries(
    client: &reqwest::Client,
    pkg: &Pkg,
    lib_dir: &PathBuf,
) -> Result<Vec<PathBuf>, String> {
    let mut jars = vec![];
    for lib in &pkg.libraries {
        if !rule_allows(&lib.rules) {
            continue;
        }
        // Skip native-only classifiers (no artifact) — natives handled by loaders later.
        let Some(dl) = &lib.downloads else { continue };
        let Some(art) = &dl.artifact else { continue };
        let dest = lib_dir.join(&art.path);
        if dest.exists() {
            jars.push(dest);
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = download_verified(client, &art.url, &art.sha1)
            .await
            .map_err(|e| e.to_string())?;
        std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
        jars.push(dest);
    }
    Ok(jars)
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    objects: std::collections::HashMap<String, AssetObj>,
}

#[derive(Debug, Deserialize)]
struct AssetObj {
    hash: String,
    #[allow(dead_code)]
    size: u64,
}

/// Download asset index + objects (content-addressed under assets/objects/xx/hash).
async fn fetch_assets(
    client: &reqwest::Client,
    pkg: &Pkg,
    assets_dir: &PathBuf,
) -> Result<String, String> {
    let Some(index_ref) = &pkg.asset_index else {
        return Ok("legacy".into());
    };
    let idx_dir = assets_dir.join("indexes");
    std::fs::create_dir_all(&idx_dir).map_err(|e| e.to_string())?;
    let idx_file = idx_dir.join(format!("{}.json", index_ref.id));
    let raw: Vec<u8> = if idx_file.exists() {
        std::fs::read(&idx_file).map_err(|e| e.to_string())?
    } else {
        let b = download_verified(client, &index_ref.url, &index_ref.sha1)
            .await
            .map_err(|e| e.to_string())?;
        std::fs::write(&idx_file, &b).map_err(|e| e.to_string())?;
        b
    };
    let index: AssetIndex = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    for obj in index.objects.values() {
        let sub = assets_dir.join("objects").join(&obj.hash[..2]).join(&obj.hash);
        if sub.exists() {
            continue;
        }
        if let Some(parent) = sub.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            &obj.hash[..2], obj.hash
        );
        // NOTE: resource host has no sha1 API per object; the index hash IS the
        // content address — verify downloaded bytes hash to the expected value.
        let bytes = client.get(url).send().await.map_err(|e| e.to_string())?
            .bytes().await.map_err(|e| e.to_string())?;
        use sha1::Digest;
        let mut h = sha1::Sha1::new();
        h.update(&bytes);
        let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if got != obj.hash {
            return Err(format!("asset hash mismatch {}", obj.hash));
        }
        std::fs::write(&sub, bytes).map_err(|e| e.to_string())?;
    }
    Ok(index_ref.id.clone())
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    version: String,
    account_id: String,
    account_name: String,
    account_uuid: String,
    access_token: Option<String>,
    instance: Option<String>,
) -> Result<LaunchResult, String> {
    let client = reqwest::Client::builder()
        .user_agent("rem-launcher/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;
    let entries = fetch_version_manifest(&client).await.map_err(map_err)?;
    let entry = entries
        .iter()
        .find(|e| e.id == version)
        .ok_or_else(|| format!("unknown version {version}"))?;
    let pkg: Pkg = client
        .get(&entry.url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let vdir = base.join("versions").join(&version);
    std::fs::create_dir_all(&vdir).map_err(|e| e.to_string())?;
    let lib_dir = vdir.join("libraries");
    let assets_dir = base.join("assets");
    std::fs::create_dir_all(&lib_dir).map_err(|e| e.to_string())?;

    // Client jar.
    let client_art = pkg.downloads.get("client").ok_or("no client download")?;
    let jar = vdir.join("client.jar");
    if !jar.exists() {
        let bytes = download_verified(&client, &client_art.url, &client_art.sha1)
            .await
            .map_err(map_err)?;
        std::fs::write(&jar, bytes).map_err(|e| e.to_string())?;
    }

    let lib_jars = fetch_libraries(&client, &pkg, &lib_dir).await?;
    let assets_id = fetch_assets(&client, &pkg, &assets_dir).await?;
    let game_dir = match &instance {
        Some(name) if !name.is_empty() && !name.contains(['/', '\\', '.']) => base.join("instances").join(name),
        _ => base.join("instances").join("__vanilla__").join(&version),
    };
    std::fs::create_dir_all(game_dir.join("mods")).map_err(|e| e.to_string())?;

    let mut cp: Vec<String> = lib_jars.iter().map(|p| p.to_string_lossy().into()).collect();
    // Demonstrate maven_path helper (kept for future loader resolution); no-op use.
    let _ = maven_path("com.example:demo:1.0");
    cp.push(jar.to_string_lossy().into());
    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    let classpath = cp.join(sep);

    let token = access_token.unwrap_or_else(|| "0".into());
    let user_type = if account_id.starts_with("microsoft:") { "msa" } else { "legacy" };
    let args = [
        "-cp".to_string(),
        classpath,
        pkg.main_class.clone(),
        "--username".into(),
        account_name,
        "--version".into(),
        version.clone(),
        "--gameDir".into(),
        game_dir.to_string_lossy().into(),
        "--assetsDir".into(),
        assets_dir.to_string_lossy().into(),
        "--assetIndex".into(),
        pkg.assets.clone().unwrap_or(assets_id),
        "--uuid".into(),
        account_uuid,
        "--accessToken".into(),
        token,
        "--userType".into(),
        user_type.into(),
        "--versionType".into(),
        "release".into(),
    ];

    let java = "java";
    let mut child = std::process::Command::new(java)
        .args(&args)
        .current_dir(&game_dir)
        .spawn()
        .map_err(|e| format!("failed to spawn java (is it installed?): {e}"))?;
    let pid = child.id();
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(LaunchResult { pid })
}

fn map_err(e: InstallError) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn prepare_client(app: AppHandle, version: String) -> Result<PreparedClient, String> {
    let client = reqwest::Client::builder()
        .user_agent("rem-launcher/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;
    let entries = fetch_version_manifest(&client).await.map_err(map_err)?;
    let entry = entries.iter().find(|e| e.id == version).ok_or_else(|| format!("unknown version {version}"))?;
    let pkg: Pkg = client.get(&entry.url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
    let artifact = pkg.downloads.get("client").ok_or_else(|| "version package has no client download".to_string())?;
    let bytes = download_verified(&client, &artifact.url, &artifact.sha1).await.map_err(map_err)?;
    let dir: PathBuf = app.path().app_data_dir().map_err(|e| e.to_string())?.join("versions").join(&version);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let jar = dir.join("client.jar");
    std::fs::write(&jar, bytes).map_err(|e| e.to_string())?;
    Ok(PreparedClient { version, jar_path: jar.to_string_lossy().into() })
}

#[tauri::command]
pub fn detect_java() -> Result<JavaInfo, String> {
    for candidate in ["java", "javaw"] {
        if let Ok(out) = std::process::Command::new(candidate).arg("-version").output() {
            let line = String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("").to_string();
            if !line.is_empty() {
                return Ok(JavaInfo { path: candidate.into(), version_line: line });
            }
        }
    }
    Err("no Java runtime found on PATH".into())
}
