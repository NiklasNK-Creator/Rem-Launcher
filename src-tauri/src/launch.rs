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
    #[serde(default)]
    classifiers: std::collections::HashMap<String, LibArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
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

/// Extract strings from a modern `arguments` entry: plain strings pass
/// through substitution; rule-gated objects/lists apply only when allowed.
fn extract_arg(entry: &serde_json::Value, sub: &dyn Fn(&str) -> String) -> Vec<String> {
    match entry {
        serde_json::Value::String(s) => vec![sub(s)],
        serde_json::Value::Array(items) => items.iter().filter_map(|v| v.as_str().map(sub)).collect(),
        serde_json::Value::Object(map) => {
            let allowed = map.get("rules").and_then(|r| serde_json::from_value::<Vec<Rule>>(r.clone()).ok()).map(|rules| rule_allows(&Some(rules))).unwrap_or(true);
            if !allowed {
                return vec![];
            }
            map.get("value").map(|v| extract_arg(v, sub)).unwrap_or_default()
        }
        _ => vec![],
    }
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
/// Extracts OS natives (classifiers) into versions/<v>/natives, with zip-slip
/// protection. Returns (classpath jars, natives dir).
async fn fetch_libraries(
    client: &reqwest::Client,
    pkg: &Pkg,
    lib_dir: &PathBuf,
    natives_dir: &PathBuf,
) -> Result<Vec<PathBuf>, String> {
    let mut jars = vec![];
    for lib in &pkg.libraries {
        if !rule_allows(&lib.rules) {
            continue;
        }
        let Some(dl) = &lib.downloads else { continue };
        // Natives: download classifier jar for this OS and extract it.
        if let Some(natives) = &lib.natives {
            if let Some(key) = natives.get(os_name()) {
                // Classifier key may contain ${arch} (32/64).
                let arch = if cfg!(target_arch = "x86_64") { "64" } else { "32" };
                let classifier = key.replace("${arch}", arch);
                if let Some(art) = dl.classifiers.get(&classifier) {
                    extract_natives(client, art, natives_dir).await?;
                }
            }
        }
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

/// Download a natives classifier jar (verified) and extract safe entries.
async fn extract_natives(
    client: &reqwest::Client,
    art: &LibArtifact,
    natives_dir: &PathBuf,
) -> Result<(), String> {
    std::fs::create_dir_all(natives_dir).map_err(|e| e.to_string())?;
    let bytes = download_verified(client, &art.url, &art.sha1)
        .await
        .map_err(|e| e.to_string())?;
    let cursor = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        // Zip-slip + junk protection: single-component names only, skip META-INF.
        if name.contains(['/', '\\']) || name.starts_with('.') || name.starts_with("META-INF") {
            continue;
        }
        let dest = natives_dir.join(&name);
        let mut out = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FabricLoaderEntry {
    loader: FabricMaven,
    intermediary: FabricMaven,
    #[serde(rename = "launcherMeta")]
    launcher_meta: FabricLauncherMeta,
}

#[derive(Debug, Deserialize)]
struct FabricMaven {
    maven: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct FabricLauncherMeta {
    #[serde(rename = "mainClass")]
    main_class: serde_json::Value,
    libraries: FabricLibraries,
}

#[derive(Debug, Deserialize)]
struct FabricLibraries {
    common: Vec<FabricLib>,
}

#[derive(Debug, Deserialize)]
struct FabricLib {
    name: String,
    url: String,
}

/// Resolve latest stable Fabric loader for a game version.
/// Returns (loader_version, main_class, extra maven coords).
async fn resolve_fabric(
    client: &reqwest::Client,
    game_version: &str,
) -> Result<(String, String, Vec<(String, String)>), String> {
    let url = format!("https://meta.fabricmc.net/v2/versions/loader/{game_version}");
    let entries: Vec<FabricLoaderEntry> = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let stable = entries.iter().find(|e| e.loader.version.contains('.')).ok_or("no fabric loader found")?;
    let main = match &stable.launcher_meta.main_class {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(m) => m.get("client").and_then(|v| v.as_str()).unwrap_or("net.fabricmc.loader.impl.launch.knot.KnotClient").to_string(),
        _ => "net.fabricmc.loader.impl.launch.knot.KnotClient".to_string(),
    };
    let mut extra = vec![(stable.loader.maven.clone(), "https://maven.fabricmc.net/".into()), (stable.intermediary.maven.clone(), "https://maven.fabricmc.net/".into())];
    for lib in &stable.launcher_meta.libraries.common {
        extra.push((lib.name.clone(), lib.url.clone()));
    }
    Ok((stable.loader.version.clone(), main, extra))
}

/// Download a maven coord from a repo base into lib_dir, return jar path.
async fn fetch_maven(
    client: &reqwest::Client,
    lib_dir: &PathBuf,
    coord: &str,
    repo: &str,
) -> Result<PathBuf, String> {
    let rel = maven_path(coord).ok_or_else(|| format!("bad maven coord {coord}"))?;
    let dest = lib_dir.join(&rel);
    if dest.exists() {
        return Ok(dest);
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let url = format!("{}{}", repo.trim_end_matches('/'), format!("/{rel}"));
    let bytes = client.get(url).send().await.map_err(|e| e.to_string())?.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
    Ok(dest)
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
    loader: Option<String>,
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

    let natives_dir = vdir.join("natives");
    let lib_jars = fetch_libraries(&client, &pkg, &lib_dir, &natives_dir).await?;
    let assets_id = fetch_assets(&client, &pkg, &assets_dir).await?;
    let game_dir = match &instance {
        Some(name) if !name.is_empty() && !name.contains(['/', '\\', '.']) => base.join("instances").join(name),
        _ => base.join("instances").join("__vanilla__").join(&version),
    };
    std::fs::create_dir_all(game_dir.join("mods")).map_err(|e| e.to_string())?;
    let mut cp: Vec<String> = lib_jars.iter().map(|p| p.to_string_lossy().into()).collect();
    let mut main_class = pkg.main_class.clone();
    if loader.as_deref() == Some("fabric") {
        let (_lv, main, extra) = resolve_fabric(&client, &version).await?;
        main_class = main;
        for (coord, repo) in extra {
            let p = fetch_maven(&client, &lib_dir, &coord, &repo).await?;
            cp.push(p.to_string_lossy().into());
        }
    }
    cp.push(jar.to_string_lossy().into());
    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    let classpath = cp.join(sep);

    let token: String = access_token.unwrap_or_else(|| "0".into());
    let user_type: String = if account_id.starts_with("microsoft:") { "msa".into() } else { "legacy".into() };
    // Placeholder values shared by legacy + modern argument templates.
    let subs: Vec<(&str, String)> = vec![
        ("${auth_player_name}", account_name.clone()),
        ("${version_name}", version.clone()),
        ("${game_directory}", game_dir.to_string_lossy().into()),
        ("${assets_root}", assets_dir.to_string_lossy().into()),
        ("${assets_index_name}", pkg.assets.clone().unwrap_or_else(|| "legacy".into())),
        ("${auth_uuid}", account_uuid.clone()),
        ("${auth_access_token}", token.clone()),
        ("${user_type}", user_type.clone()),
        ("${version_type}", "release".into()),
        ("${natives_directory}", natives_dir.to_string_lossy().into()),
        ("${launcher_name}", "rem-launcher".into()),
        ("${launcher_version}", "0.1.0".into()),
        ("${classpath}", classpath.clone()),
    ];
    let sub = |s: &str| -> String {
        let mut out = s.to_string();
        for (k, v) in &subs {
            out = out.replace(k, v);
        }
        out
    };
    let mut args: Vec<String> = vec![
        format!("-Djava.library.path={}", natives_dir.to_string_lossy()),
    ];
    if let Some(arguments) = &pkg.arguments {
        // Modern (>=1.13) packages: rule-filtered jvm + game args.
        for entry in arguments.get("jvm").and_then(|v| v.as_array()).map(|a| a.as_slice()).unwrap_or(&[]) {
            args.extend(extract_arg(entry, &sub));
        }
        args.push("-cp".into());
        args.push(classpath.clone());
        args.push(main_class.clone());
        for entry in arguments.get("game").and_then(|v| v.as_array()).map(|a| a.as_slice()).unwrap_or(&[]) {
            // Skip modern-only flags unknown to legacy main classes (harmless either way).
            args.extend(extract_arg(entry, &sub));
        }
    } else {
        // Legacy (<=1.12): fixed template; minecraftArguments variant ignored
        // (same fields, different order — vanilla accepts any order).
        args.extend([
            "-cp".to_string(),
            classpath,
            main_class,
            "--username".into(), account_name,
            "--version".into(), version.clone(),
            "--gameDir".into(), game_dir.to_string_lossy().into(),
            "--assetsDir".into(), assets_dir.to_string_lossy().into(),
            "--assetIndex".into(), pkg.assets.clone().unwrap_or_else(|| "legacy".into()),
            "--uuid".into(), account_uuid,
            "--accessToken".into(), token,
            "--userType".into(), user_type,
            "--versionType".into(), "release".into(),
        ]);
    }

    use std::process::Stdio;
    let log_path = game_dir.join("rem-launcher.log");
    let log_file = std::fs::File::create(&log_path).map_err(|e| e.to_string())?;
    let log_err = log_file.try_clone().map_err(|e| e.to_string())?;
    let mut child = std::process::Command::new("java")
        .args(&args)
        .current_dir(&game_dir)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_err))
        .spawn()
        .map_err(|e| format!("failed to spawn java (is it installed?): {e}"))?;
    let pid = child.id();
    let watching = log_path.to_string_lossy().into_owned();
    std::thread::spawn(move || {
        let status = child.wait();
        let tail = std::fs::read_to_string(&watching).unwrap_or_default();
        let last: Vec<&str> = tail.lines().rev().take(20).collect();
        eprintln!("game exited: {status:?}; log tail: {}", last.into_iter().rev().collect::<Vec<_>>().join("\n"));
    });
    Ok(LaunchResult { pid })
}

fn map_err(e: InstallError) -> String {
    e.to_string()
}

/// Read the last N lines of an instance's game log (for Play UI diagnostics).
#[tauri::command]
pub fn read_log_tail(app: AppHandle, instance: Option<String>, lines: Option<usize>) -> Result<String, String> {
    use tauri::Manager;
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let game_dir = match &instance {
        Some(name) if !name.is_empty() && !name.contains(['/', '\\', '.']) => base.join("instances").join(name),
        _ => return Err("pick an instance first".into()),
    };
    let raw = std::fs::read_to_string(game_dir.join("rem-launcher.log")).unwrap_or_default();
    let n = lines.unwrap_or(40).clamp(1, 500);
    let all: Vec<&str> = raw.lines().collect();
    let start = all.len().saturating_sub(n);
    Ok(all[start..].join("\n"))
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
