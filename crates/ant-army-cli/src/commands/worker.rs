//! Worker commands

use anyhow::Result;
use crate::client::ApiClient;

pub async fn list(_client: &ApiClient) -> Result<()> {
    // TODO: Implement
    println!("Worker list: Not yet implemented");
    Ok(())
}

pub async fn status(_client: &ApiClient, _worker_id: &str) -> Result<()> {
    // TODO: Implement
    println!("Worker status: Not yet implemented");
    Ok(())
}
