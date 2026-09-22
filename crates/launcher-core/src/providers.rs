//! Provider abstraction. See docs/architecture.md.
pub mod curseforge;
pub mod modrinth;

use crate::models::{Project, ProjectVersion, SearchQuery};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("CurseForge API key not configured")]
    MissingApiKey,
    #[error("parse error: {0}")]
    Parse(String),
    #[error("rate limited")]
    RateLimited,
}

pub type Result<T, E = ProviderError> = std::result::Result<T, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderId {
    Modrinth,
    CurseForge,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderId::Modrinth => "modrinth",
            ProviderId::CurseForge => "curseforge",
        }
    }
}

use async_trait::async_trait;

#[async_trait]
pub trait ContentProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    async fn search(&self, query: &SearchQuery) -> std::result::Result<Vec<Project>, ProviderError>;
    async fn project(&self, id: &str) -> std::result::Result<Project, ProviderError>;
    async fn versions(
        &self,
        id: &str,
        game_versions: &[String],
        loaders: &[String],
    ) -> std::result::Result<Vec<ProjectVersion>, ProviderError>;
}

pub struct ProviderRegistry {
    pub providers: Vec<Box<dyn ContentProvider>>,
}
