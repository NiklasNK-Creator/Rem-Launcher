//! Modrinth API v2 client. See docs/providers/modrinth.md.
use crate::models::*;
use crate::providers::{ContentProvider, ProviderError, ProviderId, Result};
use async_trait::async_trait;
use serde::Deserialize;
pub const BASE_URL: &str = "https://api.modrinth.com/v2";
pub const USER_AGENT: &str = "nova-launcher/0.1.0 (local-first mc launcher)";

pub struct ModrinthProvider {
    client: reqwest::Client,
    base: String,
}

impl ModrinthProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("http client"),
            base: BASE_URL.into(),
        }
    }

    #[cfg(test)]
    fn with_base(base: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("http client"),
            base: base.into(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let resp = self
            .client
            .get(format!("{}{}", self.base, path))
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited);
        }
        if !resp.status().is_success() {
            return Err(ProviderError::Http(format!("status {}", resp.status())));
        }
        resp.json::<T>()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))
    }
}

impl Default for ModrinthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct SearchResp {
    hits: Vec<Hit>,
}

#[derive(Debug, Deserialize)]
struct Hit {
    project_id: String,
    slug: Option<String>,
    title: String,
    description: String,
    project_type: String,
    versions: Vec<String>,
    #[serde(default)]
    icon_url: Option<String>,
    downloads: u64,
}

#[derive(Debug, Deserialize)]
struct FullProject {
    id: String,
    slug: Option<String>,
    title: String,
    description: String,
    project_type: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    #[serde(default)]
    icon_url: Option<String>,
    downloads: u64,
}

#[derive(Debug, Deserialize)]
struct FullVersion {
    id: String,
    project_id: String,
    name: String,
    version_number: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<FullFile>,
}

#[derive(Debug, Deserialize)]
struct FullFile {
    #[serde(default)]
    hashes: std::collections::HashMap<String, String>,
    url: String,
    filename: String,
    primary: bool,
}

fn content_type_of(s: &str) -> ContentType {
    match s {
        "mod" => ContentType::Mod,
        "resourcepack" => ContentType::ResourcePack,
        "datapack" => ContentType::DataPack,
        "shader" => ContentType::Shader,
        "modpack" => ContentType::Modpack,
        _ => ContentType::Unknown,
    }
}

fn facet_str(v: &[String]) -> Option<String> {
    if v.is_empty() {
        return None;
    }
    let inner: Vec<String> = v.iter().map(|s| format!("\"{s}\"")).collect();
    Some(format!("[{}]", inner.join(",")))
}

#[async_trait]
impl ContentProvider for ModrinthProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Modrinth
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<Project>> {
        let mut facets: Vec<String> = vec![];
        if let Some(t) = &query.content_type {
            let pt = match t {
                ContentType::Mod => "mod",
                ContentType::ResourcePack => "resourcepack",
                ContentType::DataPack => "datapack",
                ContentType::Shader => "shader",
                ContentType::Modpack => "modpack",
                ContentType::Unknown => "",
            };
            if !pt.is_empty() {
                facets.push(format!("[\"project_type:{pt}\"]"));
            }
        }
        if let Some(f) = facet_str(&query.game_versions) {
            let vs: Vec<String> =
                query.game_versions.iter().map(|v| format!("\"versions:{v}\"")).collect();
            facets.push(format!("[{}]", vs.join(",")));
            let _ = f;
        }
        if !query.loaders.is_empty() {
            let ls: Vec<String> =
                query.loaders.iter().map(|v| format!("\"categories:{v}\"")).collect();
            facets.push(format!("[{}]", ls.join(",")));
        }
        let mut url = format!(
            "/search?query={}&limit={}&offset={}",
            urlencoding::encode(&query.text),
            query.limit.max(1).min(100),
            query.offset
        );
        if !facets.is_empty() {
            url.push_str(&format!("&facets={}", urlencoding::encode(&format!("[{}]", facets.join(",")))));
        }
        let resp: SearchResp = self.get(&url).await?;
        Ok(resp
            .hits
            .into_iter()
            .map(|h| Project {
                provider: ProviderId::Modrinth.as_str().into(),
                id: h.project_id,
                slug: h.slug,
                title: h.title,
                description: h.description,
                content_type: content_type_of(&h.project_type),
                game_versions: h.versions,
                loaders: vec![],
                icon_url: h.icon_url,
                downloads: h.downloads,
            })
            .collect())
    }

    async fn project(&self, id: &str) -> Result<Project> {
        let p: FullProject = self.get(&format!("/project/{id}")).await?;
        Ok(Project {
            provider: ProviderId::Modrinth.as_str().into(),
            id: p.id,
            slug: p.slug,
            title: p.title,
            description: p.description,
            content_type: content_type_of(&p.project_type),
            game_versions: p.game_versions,
            loaders: p.loaders,
            icon_url: p.icon_url,
            downloads: p.downloads,
        })
    }

    async fn versions(
        &self,
        id: &str,
        game_versions: &[String],
        loaders: &[String],
    ) -> Result<Vec<ProjectVersion>> {
        let mut url = format!("/project/{id}/version?");
        if !game_versions.is_empty() {
            url.push_str(&format!(
                "game_versions={}",
                urlencoding::encode(&serde_json::to_string(game_versions).unwrap())
            ));
        }
        if !loaders.is_empty() {
            if !url.ends_with('?') {
                url.push('&');
            }
            url.push_str(&format!(
                "loaders={}",
                urlencoding::encode(&serde_json::to_string(loaders).unwrap())
            ));
        }
        let vs: Vec<FullVersion> = self.get(&url).await?;
        Ok(vs
            .into_iter()
            .map(|v| ProjectVersion {
                id: v.id,
                project_id: v.project_id,
                name: v.name,
                version_number: v.version_number,
                game_versions: v.game_versions,
                loaders: v.loaders,
                files: v
                    .files
                    .into_iter()
                    .map(|f| VersionFile {
                        name: f.filename,
                        url: f.url,
                        sha512: f.hashes.get("sha512").cloned(),
                        primary: f.primary,
                    })
                    .collect(),
            })
            .collect())
    }
}
