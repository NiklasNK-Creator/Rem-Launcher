//! CurseForge Core client (key-gated skeleton). See docs/providers/curseforge.md.
use crate::models::{Project, ProjectVersion, SearchQuery};
use crate::providers::{ContentProvider, ProviderError, ProviderId, Result};
use async_trait::async_trait;

pub struct CurseForgeProvider {
    api_key: Option<String>,
}

impl CurseForgeProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    pub fn from_env() -> Self {
        Self::new(std::env::var("CURSEFORGE_API_KEY").ok())
    }

    fn require_key(&self) -> Result<&str> {
        self.api_key.as_deref().ok_or(ProviderError::MissingApiKey)
    }
}

#[async_trait]
impl ContentProvider for CurseForgeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::CurseForge
    }

    async fn search(&self, _query: &SearchQuery) -> Result<Vec<Project>> {
        let _ = self.require_key()?;
        // Full HTTP mapping is next step after key strategy is decided.
        Ok(vec![])
    }

    async fn project(&self, _id: &str) -> Result<Project> {
        let _ = self.require_key()?;
        Ok(Project {
            provider: ProviderId::CurseForge.as_str().into(),
            id: _id.into(),
            slug: None,
            title: String::new(),
            description: String::new(),
            content_type: crate::models::ContentType::Unknown,
            game_versions: vec![],
            loaders: vec![],
            icon_url: None,
            downloads: 0,
            follows: 0,
            updated: None,
        })
    }

    async fn versions(
        &self,
        _id: &str,
        _game_versions: &[String],
        _loaders: &[String],
    ) -> Result<Vec<ProjectVersion>> {
        let _ = self.require_key()?;
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_key_errors() {
        let p = CurseForgeProvider::new(None);
        let q = SearchQuery::default();
        assert!(matches!(p.search(&q).await, Err(ProviderError::MissingApiKey)));
    }
}
