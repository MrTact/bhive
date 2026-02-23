//! LLM provider abstraction

use crate::types::ProviderConfig;
use crate::Result;

// TODO: Implement provider abstraction using rust-genai
// This is a stub for now

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider from configuration
    /// TODO: Implement actual provider creation using rust-genai
    #[allow(dead_code)]
    pub fn create(_config: &ProviderConfig) -> Result<()> {
        // Stub - will implement with rust-genai integration
        Ok(())
    }
}
