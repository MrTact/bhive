//! Worker naming system
//!
//! Generates unique worker names in the format `<adjective>-<noun>`
//! (e.g., "swift-falcon", "clever-badger").
//!
//! Names are loaded from `<project_root>/.config/bhive/names.toml` and can be
//! customized per-project. A default template is copied on `bhive init`.

use crate::Result;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Default names.toml embedded in the binary
pub const DEFAULT_NAMES_TOML: &str = include_str!("../resources/names.toml");

/// Configuration for worker naming loaded from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    /// List of adjectives
    pub adjectives: Vec<String>,
    /// List of nouns
    pub nouns: Vec<String>,
}

impl Default for NamingConfig {
    fn default() -> Self {
        // Parse the embedded default config
        toml::from_str(DEFAULT_NAMES_TOML)
            .expect("embedded names.toml should be valid")
    }
}

impl NamingConfig {
    /// Load naming config from a TOML file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            crate::Error::Config(format!("Failed to read names.toml: {}", e))
        })?;
        
        let config: NamingConfig = toml::from_str(&contents).map_err(|e| {
            crate::Error::Config(format!("Failed to parse names.toml: {}", e))
        })?;
        
        Ok(config)
    }

    /// Load from a project's config directory, falling back to embedded defaults
    pub fn load_for_project(project_root: &Path) -> Self {
        let config_path = Self::project_path(project_root);
        
        if config_path.exists() {
            match Self::load_from_file(&config_path) {
                Ok(config) => {
                    tracing::debug!("Loaded naming config from {:?}", config_path);
                    return config;
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load naming config for project {:?}, using defaults: {}",
                        project_root, e
                    );
                }
            }
        } else {
            tracing::debug!(
                "No names.toml found at {:?}, using embedded defaults",
                config_path
            );
        }
        
        Self::default()
    }

    /// Get the config file path for a project
    pub fn project_path(project_root: &Path) -> PathBuf {
        project_root
            .join(".config")
            .join("bhive")
            .join("names.toml")
    }

    /// Copy the default names.toml to a project's config directory
    /// 
    /// Used by `bhive init` to create user-customizable config.
    pub fn install_for_project(project_root: &Path) -> Result<PathBuf> {
        let path = Self::project_path(project_root);
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Config(format!("Failed to create config directory: {}", e))
            })?;
        }
        
        std::fs::write(&path, DEFAULT_NAMES_TOML).map_err(|e| {
            crate::Error::Config(format!("Failed to write names.toml: {}", e))
        })?;
        
        Ok(path)
    }

    /// Total possible unique combinations
    pub fn total_combinations(&self) -> usize {
        self.adjectives.len() * self.nouns.len()
    }
}

/// Worker name generator for a single project
#[derive(Debug, Clone)]
pub struct WorkerNameGenerator {
    config: NamingConfig,
}

impl WorkerNameGenerator {
    /// Create a new generator with the given config
    pub fn new(config: NamingConfig) -> Self {
        Self { config }
    }

    /// Create a generator for a specific project
    pub fn for_project(project_root: &Path) -> Self {
        Self::new(NamingConfig::load_for_project(project_root))
    }

    /// Create a generator with embedded defaults (no file loading)
    pub fn with_defaults() -> Self {
        Self::new(NamingConfig::default())
    }

    /// Generate a random name
    pub fn generate(&self) -> String {
        let mut rng = rand::thread_rng();
        
        let adjective = self.config.adjectives
            .choose(&mut rng)
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        
        let noun = self.config.nouns
            .choose(&mut rng)
            .map(|s| s.as_str())
            .unwrap_or("worker");
        
        format!("{}-{}", adjective, noun)
    }

    /// Generate a unique name not in the existing set
    /// 
    /// Returns `None` if unable to generate a unique name after max_attempts
    pub fn generate_unique(&self, existing: &HashSet<String>, max_attempts: usize) -> Option<String> {
        for _ in 0..max_attempts {
            let name = self.generate();
            if !existing.contains(&name) {
                return Some(name);
            }
        }
        None
    }

    /// Generate a unique name, checking against an async predicate function
    /// 
    /// Useful when checking against database
    pub async fn generate_unique_with<F, Fut>(&self, exists_fn: F, max_attempts: usize) -> Option<String>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        for _ in 0..max_attempts {
            let name = self.generate();
            if !exists_fn(name.clone()).await {
                return Some(name);
            }
        }
        None
    }

    /// Get total possible unique combinations
    pub fn total_combinations(&self) -> usize {
        self.config.total_combinations()
    }

    /// Get reference to the underlying config
    pub fn config(&self) -> &NamingConfig {
        &self.config
    }
}

impl Default for WorkerNameGenerator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Cache of name generators per project
/// 
/// The Queen uses this to lazy-load naming configs per project.
#[derive(Debug, Default)]
pub struct NamingService {
    /// Cached generators by project root path
    cache: RwLock<HashMap<PathBuf, Arc<WorkerNameGenerator>>>,
}

impl NamingService {
    /// Create a new naming service
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a name generator for a project
    /// 
    /// Lazy-loads the project's names.toml on first access.
    pub fn generator_for(&self, project_root: &Path) -> Arc<WorkerNameGenerator> {
        // Try read lock first (fast path)
        {
            let cache = self.cache.read().unwrap();
            if let Some(gen) = cache.get(project_root) {
                return gen.clone();
            }
        }

        // Need to create - take write lock
        let mut cache = self.cache.write().unwrap();
        
        // Double-check in case another thread created it
        if let Some(gen) = cache.get(project_root) {
            return gen.clone();
        }

        // Create and cache
        let generator = Arc::new(WorkerNameGenerator::for_project(project_root));
        cache.insert(project_root.to_path_buf(), generator.clone());
        
        tracing::info!(
            "Loaded naming config for project {:?} ({} combinations)",
            project_root,
            generator.total_combinations()
        );
        
        generator
    }

    /// Invalidate cached generator for a project (e.g., after config change)
    pub fn invalidate(&self, project_root: &Path) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(project_root);
    }

    /// Clear all cached generators
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_config_valid() {
        // This will panic if the embedded TOML is invalid
        let config = NamingConfig::default();
        assert!(!config.adjectives.is_empty(), "Should have adjectives");
        assert!(!config.nouns.is_empty(), "Should have nouns");
    }

    #[test]
    fn test_generate_name_format() {
        let generator = WorkerNameGenerator::with_defaults();
        let name = generator.generate();
        
        assert!(name.contains('-'), "Name should contain a hyphen: {}", name);
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 2, "Name should have exactly 2 parts: {}", name);
        assert!(!parts[0].is_empty(), "Adjective should not be empty");
        assert!(!parts[1].is_empty(), "Noun should not be empty");
    }

    #[test]
    fn test_generate_unique() {
        let generator = WorkerNameGenerator::with_defaults();
        let mut existing = HashSet::new();
        
        // Generate 100 unique names
        for _ in 0..100 {
            let name = generator.generate_unique(&existing, 100).unwrap();
            assert!(!existing.contains(&name), "Name should be unique");
            existing.insert(name);
        }
        
        assert_eq!(existing.len(), 100);
    }

    #[test]
    fn test_total_combinations() {
        let generator = WorkerNameGenerator::with_defaults();
        let combos = generator.total_combinations();
        
        // Should have a good number of combinations
        assert!(combos >= 10_000, "Should have at least 10,000 combinations, got {}", combos);
        println!("Total combinations: {}", combos);
    }

    #[test]
    fn test_config_words_are_lowercase() {
        let config = NamingConfig::default();
        
        for adj in &config.adjectives {
            assert!(
                adj.chars().all(|c| c.is_ascii_lowercase()),
                "Adjective should be lowercase: {}", adj
            );
        }
        for noun in &config.nouns {
            assert!(
                noun.chars().all(|c| c.is_ascii_lowercase()),
                "Noun should be lowercase: {}", noun
            );
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = NamingConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: NamingConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.adjectives.len(), parsed.adjectives.len());
        assert_eq!(config.nouns.len(), parsed.nouns.len());
    }

    #[test]
    fn test_default_toml_content() {
        // Verify the embedded TOML has expected structure
        assert!(DEFAULT_NAMES_TOML.contains("adjectives"));
        assert!(DEFAULT_NAMES_TOML.contains("nouns"));
        assert!(DEFAULT_NAMES_TOML.contains("swift"));
        assert!(DEFAULT_NAMES_TOML.contains("falcon"));
    }

    #[test]
    fn test_naming_service_caching() {
        let service = NamingService::new();
        let project_root = PathBuf::from("/tmp/test-project");
        
        // First access should create and cache
        let gen1 = service.generator_for(&project_root);
        let gen2 = service.generator_for(&project_root);
        
        // Should be the same Arc (same pointer)
        assert!(Arc::ptr_eq(&gen1, &gen2), "Should return cached generator");
        
        // Invalidate and get again - should be different Arc
        service.invalidate(&project_root);
        let gen3 = service.generator_for(&project_root);
        assert!(!Arc::ptr_eq(&gen1, &gen3), "Should create new generator after invalidate");
    }

    #[test]
    fn test_project_path() {
        let project_root = PathBuf::from("/home/user/my-project");
        let config_path = NamingConfig::project_path(&project_root);
        
        assert_eq!(
            config_path,
            PathBuf::from("/home/user/my-project/.config/bhive/names.toml")
        );
    }
}
