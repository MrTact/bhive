//! Project registry management

use super::ProjectConfig;
use crate::{Error, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Global project registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectRegistry {
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

/// Information about an orphaned project
#[derive(Debug, Clone)]
pub struct OrphanedProject {
    pub project_id: String,
    pub original_path: PathBuf,
    pub db_name: String,
    pub qdrant_collection: String,
}

impl ProjectRegistry {
    /// Load registry from ~/.config/bhive/projects.toml
    pub fn load() -> Result<Self> {
        let path = Self::registry_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)?;
        let registry: Self = toml::from_str(&contents)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to parse registry: {}", e)))?;

        Ok(registry)
    }

    /// Save registry to ~/.config/bhive/projects.toml
    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(&self)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to serialize registry: {}", e)))?;

        fs::write(&path, contents)?;

        Ok(())
    }

    /// Register a new project
    pub fn register(&mut self, config: ProjectConfig) -> Result<()> {
        let project_id = config.project_id.clone();
        self.projects.insert(project_id, config);
        self.save()?;
        Ok(())
    }

    /// Get project config by path
    pub fn get_by_path(&self, path: &Path) -> Option<&ProjectConfig> {
        let path = path.canonicalize().ok()?;
        self.projects
            .values()
            .find(|p| p.path.canonicalize().ok().as_ref() == Some(&path))
    }

    /// Get project config by project ID
    pub fn get_by_id(&self, project_id: &str) -> Option<&ProjectConfig> {
        self.projects.get(project_id)
    }

    /// Update last_seen timestamp for a project
    pub fn update_last_seen(&mut self, path: &Path) -> Result<()> {
        if let Some(config) = self.get_by_path_mut(path) {
            config.last_seen = Utc::now();
            self.save()?;
        }
        Ok(())
    }

    /// Remove project from registry (keep data)
    pub fn unregister(&mut self, path: &Path) -> Result<bool> {
        let path = path
            .canonicalize()
            .map_err(|e| Error::Other(anyhow::anyhow!("Invalid path: {}", e)))?;

        let project_id = self
            .projects
            .values()
            .find(|p| p.path.canonicalize().ok().as_ref() == Some(&path))
            .map(|p| p.project_id.clone());

        if let Some(project_id) = project_id {
            self.projects.remove(&project_id);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Find orphaned projects (directory no longer exists)
    pub fn find_orphans(&self) -> Vec<OrphanedProject> {
        self.projects
            .values()
            .filter(|config| !config.directory_exists())
            .map(|config| OrphanedProject {
                project_id: config.project_id.clone(),
                original_path: config.path.clone(),
                db_name: config.db_name.clone(),
                qdrant_collection: config.qdrant_collection.clone(),
            })
            .collect()
    }

    /// List all projects
    pub fn list(&self) -> Vec<&ProjectConfig> {
        let mut projects: Vec<_> = self.projects.values().collect();
        projects.sort_by_key(|p| &p.last_seen);
        projects.reverse(); // Most recent first
        projects
    }

    /// Check if project exists in registry
    pub fn contains(&self, path: &Path) -> bool {
        self.get_by_path(path).is_some()
    }

    /// Get mutable reference by path
    fn get_by_path_mut(&mut self, path: &Path) -> Option<&mut ProjectConfig> {
        let path = path.canonicalize().ok()?;
        self.projects
            .values_mut()
            .find(|p| p.path.canonicalize().ok().as_ref() == Some(&path))
    }

    /// Get path to registry file
    fn registry_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| Error::Other(anyhow::anyhow!("Could not determine home directory")))?;

        Ok(home.join(".config/bhive/projects.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_registry_serialization() {
        let mut registry = ProjectRegistry::default();

        let config = ProjectConfig::new("/tmp/test-project");
        registry.projects.insert(config.project_id.clone(), config);

        let toml_str = toml::to_string(&registry).unwrap();
        assert!(toml_str.contains("[projects."));
        assert!(toml_str.contains("project_id"));
        assert!(toml_str.contains("db_name"));
    }

    #[test]
    fn test_find_orphans() {
        let mut registry = ProjectRegistry::default();

        // Add existing directory
        let existing = env::temp_dir();
        let config1 = ProjectConfig::new(&existing);
        registry.projects.insert(config1.project_id.clone(), config1);

        // Add non-existent directory
        let non_existent = PathBuf::from("/this/does/not/exist");
        let config2 = ProjectConfig::new(&non_existent);
        registry.projects.insert(config2.project_id.clone(), config2);

        let orphans = registry.find_orphans();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].original_path, non_existent);
    }
}
