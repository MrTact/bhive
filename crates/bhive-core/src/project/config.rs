//! Project configuration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for a single project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Stable project identifier (derived from path)
    pub project_id: String,

    /// Absolute path to project root
    pub path: PathBuf,

    /// PostgreSQL database name
    pub db_name: String,

    /// Qdrant collection name (for LEGOMem)
    pub qdrant_collection: String,

    /// Redis key prefix
    pub redis_prefix: String,

    /// When the project was registered
    pub created_at: DateTime<Utc>,

    /// Last time bhive ran in this project
    pub last_seen: DateTime<Utc>,
}

impl ProjectConfig {
    /// Create new project config from path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let project_id = Self::generate_project_id(&path);
        let now = Utc::now();

        Self {
            project_id: project_id.clone(),
            path,
            db_name: format!("bhive_{}", project_id),
            qdrant_collection: format!("legomem_{}", project_id),
            redis_prefix: format!("{}:", Self::short_id(&project_id)),
            created_at: now,
            last_seen: now,
        }
    }

    /// Generate stable project ID from path
    /// Format: {dir_name}_{hash} (e.g., "my_app_a1b2")
    fn generate_project_id(path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Get directory name
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");

        // Sanitize: lowercase, replace non-alphanumeric with underscore
        let sanitized = dir_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>();

        // Hash the full absolute path for uniqueness
        let mut hasher = DefaultHasher::new();
        path.to_string_lossy().hash(&mut hasher);
        let hash = hasher.finish();

        // Take first 4 hex chars of hash
        let hash_suffix = format!("{:x}", hash).chars().take(4).collect::<String>();

        format!("{}_{}", sanitized, hash_suffix)
    }

    /// Get short ID for prefixes (first 6 chars)
    fn short_id(project_id: &str) -> String {
        project_id.chars().take(6).collect()
    }

    /// Get database connection URL
    pub fn database_url(&self, password: &str) -> String {
        format!(
            "postgresql://bhive:{}@localhost:5432/{}",
            password, self.db_name
        )
    }

    /// Check if project directory exists
    pub fn directory_exists(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_project_id() {
        let path = PathBuf::from("/Users/tkeating/projects/my-app");
        let config = ProjectConfig::new(&path);

        assert!(config.project_id.starts_with("my_app_"));
        assert_eq!(config.project_id.len(), "my_app_".len() + 4); // name + 4 char hash
    }

    #[test]
    fn test_database_name() {
        let config = ProjectConfig::new("/Users/tkeating/projects/test");
        assert!(config.db_name.starts_with("bhive_"));
    }

    #[test]
    fn test_stable_id_for_same_path() {
        let path = PathBuf::from("/Users/tkeating/projects/my-app");
        let config1 = ProjectConfig::new(&path);
        let config2 = ProjectConfig::new(&path);

        assert_eq!(config1.project_id, config2.project_id);
    }
}
