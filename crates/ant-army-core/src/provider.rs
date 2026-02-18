//! LLM provider abstraction

use crate::types::ProviderConfig;
use crate::{Error, Result};

/// Trait for LLM providers
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Generate a response
    async fn generate(&self, prompt: &str) -> Result<String>;

    /// Stream a response (optional, default uses generate)
    async fn stream(&self, prompt: &str) -> Result<tokio::sync::mpsc::Receiver<String>> {
        let response = self.generate(prompt).await?;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            let _ = tx.send(response).await;
        });
        Ok(rx)
    }
}

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider from configuration
    pub fn create(_config: &ProviderConfig) -> Result<Box<dyn Provider>> {
        // TODO: Implement actual provider creation using rust-genai
        // For now, return a stub
        Err(Error::ProviderError(
            "Provider creation not yet implemented".to_string(),
        ))
    }
}
