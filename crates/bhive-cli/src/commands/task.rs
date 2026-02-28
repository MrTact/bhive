//! Task commands

use anyhow::Result;
use bhive_core::task::{CreateTaskRequest, CreateTaskResponse, TaskProviderConfig};
use std::collections::HashMap;

use crate::client::ApiClient;

pub async fn create(
    client: &ApiClient,
    description: String,
    files: Vec<String>,
    max_workers: u32,
    generate: String,
    review: Option<String>,
) -> Result<()> {
    let request = CreateTaskRequest {
        description: description.clone(),
        files,
        max_workers: Some(max_workers),
        providers: TaskProviderConfig { generate, review },
        metadata: HashMap::new(),
    };

    let response: CreateTaskResponse = client.post("/api/v1/tasks", &request).await?;

    println!("✓ Task created successfully");
    println!("  Task ID: {}", response.task_id);
    println!("  Status: {}", response.status);
    println!("  Description: {}", description);
    println!("\nUse 'bhive task watch {}' to monitor progress", response.task_id);

    Ok(())
}

pub async fn status(_client: &ApiClient, _task_id: &str) -> Result<()> {
    // TODO: Implement
    println!("Task status: Not yet implemented");
    Ok(())
}

pub async fn watch(_client: &ApiClient, _task_id: &str) -> Result<()> {
    // TODO: Implement SSE streaming
    println!("Task watch: Not yet implemented");
    Ok(())
}

pub async fn list(_client: &ApiClient) -> Result<()> {
    // TODO: Implement
    println!("Task list: Not yet implemented");
    Ok(())
}
