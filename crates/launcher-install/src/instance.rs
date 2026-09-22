//! Instance model: on-disk layout + lockfile.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub dir: PathBuf,
}

impl Instance {
    pub fn mods_dir(&self) -> PathBuf {
        self.dir.join("mods")
    }
    /// Reject path traversal: file name must be a single normal component.
    pub fn safe_mod_path(&self, file_name: &str) -> Option<PathBuf> {
        let p = std::path::Path::new(file_name);
        if p.components().count() != 1 {
            return None;
        }
        if file_name.is_empty() || file_name.starts_with('.') {
            return None;
        }
        Some(self.mods_dir().join(file_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_traversal() {
        let i = Instance {
            name: "t".into(),
            game_version: "1.20.1".into(),
            loader: "fabric".into(),
            dir: PathBuf::from("/data/t"),
        };
        assert!(i.safe_mod_path("../evil.jar").is_none());
        assert!(i.safe_mod_path("a/b.jar").is_none());
        assert!(i.safe_mod_path("sodium.jar").is_some());
    }
}
