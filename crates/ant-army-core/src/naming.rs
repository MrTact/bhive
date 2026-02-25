//! Ant naming system
//!
//! Generates unique ant names in the format `<adjective>-<noun>`
//! (e.g., "swift-falcon", "clever-badger").
//!
//! Names are loaded from `~/.config/ant-army/names.toml` and can be
//! customized by the user.

use crate::Result;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Default word lists embedded in the binary
pub mod defaults {
    pub const ADJECTIVES: &[&str] = &[
        "swift", "clever", "bold", "quiet", "bright", "steady", "keen", "nimble", "wise", "brave",
        "eager", "gentle", "hardy", "jolly", "lively", "merry", "noble", "proud", "quick", "sharp",
        "agile", "alert", "calm", "daring", "earnest", "fair", "grand", "honest", "iron", "just",
        "kind", "loyal", "mighty", "neat", "open", "patient", "ready", "silent", "true", "vivid",
        "warm", "young", "zesty", "able", "active", "astute", "blithe", "cosmic", "crisp", "deft",
        "eager", "exact", "fierce", "fleet", "frank", "fresh", "glad", "golden", "great", "happy",
        "humble", "ideal", "jade", "keen", "laser", "lucid", "lunar", "major", "mellow", "mint",
        "modern", "mossy", "native", "novel", "olive", "onyx", "optimal", "orange", "outer", "pale",
        "pearl", "pink", "plain", "plum", "polar", "prime", "pure", "radiant", "rapid", "rare",
        "regal", "rich", "robust", "rosy", "royal", "ruby", "rustic", "sage", "satin", "savvy",
        "scarlet", "serene", "silver", "simple", "sleek", "smart", "smooth", "snowy", "solar",
        "solid", "sonic", "spiral", "spring", "stable", "stark", "steady", "steel", "stellar",
        "stoic", "storm", "strong", "subtle", "summer", "sunny", "super", "sweet", "tan", "teal",
        "tender", "terra", "tidal", "timber", "topaz", "tough", "tranquil", "trim", "turbo",
        "twilight", "ultra", "unified", "unique", "upbeat", "urban", "useful", "valid", "valor",
        "velvet", "verdant", "vibrant", "violet", "vital", "vivid", "wary", "wild", "winter",
        "witty", "wooden", "worthy", "xenial", "yellow", "young", "zealous", "zen", "zephyr",
        "zinc", "zippy", "azure", "amber", "arctic", "autumn", "binary", "blazing", "breezy",
        "bronze", "carbon", "cedar", "cherry", "chrome", "cipher", "citrus", "classic", "clear",
        "clever", "cliff", "cloud", "cobalt", "copper", "coral", "cosmic", "cotton", "crystal",
        "cyber", "dawn", "delta", "desert", "diamond", "digital", "distant", "dotted", "dusk",
        "dusty", "dynamic", "echo", "edge", "ember", "emerald", "epic", "eternal", "evening",
    ];

    pub const NOUNS: &[&str] = &[
        "falcon", "badger", "otter", "raven", "fox", "eagle", "wolf", "bear", "hawk", "lynx",
        "tiger", "lion", "puma", "jaguar", "panther", "cobra", "viper", "python", "crane", "heron",
        "owl", "sparrow", "finch", "robin", "wren", "dove", "swan", "goose", "duck", "pelican",
        "salmon", "trout", "bass", "pike", "carp", "whale", "dolphin", "shark", "seal", "walrus",
        "moose", "elk", "deer", "stag", "bison", "buffalo", "rhino", "hippo", "zebra", "giraffe",
        "koala", "panda", "lemur", "sloth", "gecko", "iguana", "turtle", "tortoise", "frog", "toad",
        "ant", "bee", "wasp", "hornet", "beetle", "mantis", "cricket", "moth", "butterfly", "dragonfly",
        "acorn", "aspen", "bamboo", "birch", "cedar", "cypress", "elm", "fern", "grove", "hazel",
        "ivy", "jasmine", "juniper", "laurel", "lotus", "maple", "oak", "olive", "orchid", "palm",
        "pine", "poplar", "redwood", "rose", "sage", "sequoia", "spruce", "thistle", "tulip", "willow",
        "arrow", "anchor", "anvil", "beacon", "blade", "bolt", "bridge", "cannon", "castle", "chain",
        "chariot", "cipher", "citadel", "comet", "compass", "crown", "crystal", "dagger", "diamond",
        "dome", "dragon", "eclipse", "ember", "engine", "falcon", "feather", "flame", "flare", "forge",
        "frost", "galaxy", "garden", "gate", "glacier", "globe", "hammer", "harbor", "herald", "horizon",
        "icon", "island", "jade", "javelin", "jewel", "keystone", "knight", "lance", "lantern", "laser",
        "ledger", "legend", "lightning", "lotus", "marble", "meadow", "meteor", "mirror", "mist",
        "moon", "mountain", "nebula", "nexus", "oasis", "ocean", "oracle", "orbit", "peak", "pearl",
        "phantom", "phoenix", "pillar", "pioneer", "pixel", "planet", "plasma", "prism", "pulse",
        "pyramid", "quartz", "quest", "rainbow", "rapids", "reef", "ridge", "river", "rocket", "rune",
        "saber", "sail", "sapphire", "scroll", "shadow", "shield", "signal", "silk", "sky", "spark",
        "spear", "sphere", "spirit", "spring", "star", "steam", "stone", "storm", "stream", "summit",
        "sun", "sword", "temple", "thunder", "tide", "titan", "torch", "tower", "trail", "trident",
    ];
}

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
        Self {
            adjectives: defaults::ADJECTIVES.iter().map(|s| (*s).to_string()).collect(),
            nouns: defaults::NOUNS.iter().map(|s| (*s).to_string()).collect(),
        }
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

    /// Load from default location (~/.config/ant-army/names.toml) or use defaults
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

    /// Save the default config to a file (for `ant-army init`)
    pub fn save_default_to_file(path: &Path) -> Result<()> {
        let config = Self::default();
        let contents = toml::to_string_pretty(&config).map_err(|e| {
            crate::Error::Config(format!("Failed to serialize names.toml: {}", e))
        })?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Config(format!("Failed to create config directory: {}", e))
            })?;
        }
        
        std::fs::write(path, contents).map_err(|e| {
            crate::Error::Config(format!("Failed to write names.toml: {}", e))
        })?;
        
        Ok(())
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

    /// Generate a unique name, checking against a predicate function
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
        
        // Should have at least 40,000 combinations (200 * 200)
        assert!(combos >= 40_000, "Should have at least 40,000 combinations, got {}", combos);
    }

    #[test]
    fn test_default_config_valid() {
        let config = NamingConfig::default();
        
        assert!(!config.adjectives.is_empty(), "Should have adjectives");
        assert!(!config.nouns.is_empty(), "Should have nouns");
        
        // All words should be lowercase and contain only letters
        for adj in &config.adjectives {
            assert!(adj.chars().all(|c| c.is_ascii_lowercase()), 
                "Adjective should be lowercase: {}", adj);
        }
        for noun in &config.nouns {
            assert!(noun.chars().all(|c| c.is_ascii_lowercase()), 
                "Noun should be lowercase: {}", noun);
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
}
