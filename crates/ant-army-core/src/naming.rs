//! Ant naming system
//!
//! Generates unique ant names in the format `<adjective>-<noun>`
//! (e.g., "swift-falcon", "clever-badger").
//!
//! Names are loaded from `~/.config/ant-army/names.toml` and can be
//! customized by the user. A default template is copied on `ant-army init`.

use crate::Result;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Default names.toml embedded in the binary
pub const DEFAULT_NAMES_TOML: &str = include_str!("../resources/names.toml");

/// Configuration for ant naming loaded from TOML
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

    /// Load from default location (~/.config/ant-army/names.toml) or use embedded defaults
    pub fn load_or_default() -> Self {
        let config_path = Self::default_path();
        
        if config_path.exists() {
            match Self::load_from_file(&config_path) {
                Ok(config) => {
                    tracing::debug!("Loaded naming config from {:?}", config_path);
                    return config;
                }
                Err(e) => {
                    tracing::warn!("Failed to load naming config, using defaults: {}", e);
                }
            }
        }
        
        Self::default()
    }

    /// Get the default config file path
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ant-army")
            .join("names.toml")
    }

    /// Copy the default names.toml to the specified path
    /// 
    /// Used by `ant-army init` to create user-customizable config.
    pub fn copy_default_to(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Config(format!("Failed to create config directory: {}", e))
            })?;
        }
        
        std::fs::write(path, DEFAULT_NAMES_TOML).map_err(|e| {
            crate::Error::Config(format!("Failed to write names.toml: {}", e))
        })?;
        
        Ok(())
    }

    /// Copy the default names.toml to the default location
    pub fn install_default() -> Result<PathBuf> {
        let path = Self::default_path();
        Self::copy_default_to(&path)?;
        Ok(path)
    }

    /// Total possible unique combinations
    pub fn total_combinations(&self) -> usize {
        self.adjectives.len() * self.nouns.len()
    }
}

/// Ant name generator
#[derive(Debug, Clone)]
pub struct AntNameGenerator {
    config: NamingConfig,
}

impl AntNameGenerator {
    /// Create a new generator with the given config
    pub fn new(config: NamingConfig) -> Self {
        Self { config }
    }

    /// Create a generator with default config (or loaded from file)
    pub fn with_defaults() -> Self {
        Self::new(NamingConfig::load_or_default())
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
            .unwrap_or("ant");
        
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

impl Default for AntNameGenerator {
    fn default() -> Self {
        Self::with_defaults()
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
        let generator = AntNameGenerator::with_defaults();
        let name = generator.generate();
        
        assert!(name.contains('-'), "Name should contain a hyphen: {}", name);
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 2, "Name should have exactly 2 parts: {}", name);
        assert!(!parts[0].is_empty(), "Adjective should not be empty");
        assert!(!parts[1].is_empty(), "Noun should not be empty");
    }

    #[test]
    fn test_generate_unique() {
        let generator = AntNameGenerator::with_defaults();
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
        let generator = AntNameGenerator::with_defaults();
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
}
