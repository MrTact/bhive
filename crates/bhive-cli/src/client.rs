//! API client for communicating with the B'hive service

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct ApiClient {
    base_url: String,
    client: Client,
    project_id: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
            project_id: None,
        }
    }

    pub fn with_project_id(mut self, project_id: String) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.post(&url).json(body);

        // Add project ID header if present
        if let Some(ref project_id) = self.project_id {
            request = request.header("X-Project-ID", project_id);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error: {} - {}", status, body);
        }

        let result = response.json().await?;
        Ok(result)
    }

    pub async fn get<R: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);

        // Add project ID header if present
        if let Some(ref project_id) = self.project_id {
            request = request.header("X-Project-ID", project_id);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error: {} - {}", status, body);
        }

        let result = response.json().await?;
        Ok(result)
    }
}
