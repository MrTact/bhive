//! Project detection and validation

use anyhow::{Context, Result};
use bhive_core::project::{ProjectConfig, ProjectRegistry};
use std::env;

/// Get the current project configuration
pub fn get_current_project() -> Result<ProjectConfig> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let registry = ProjectRegistry::load().context(
        "Failed to load project registry. Have you run 'bhive init' yet?",
    )?;

    let project = registry
        .get_by_path(&current_dir)
        .context(format!(
            "Project not initialized in this directory.\n\n\
             Current directory: {}\n\n\
             To initialize bhive for this project, run:\n  \
             bhive init",
            current_dir.display()
        ))?;

    Ok(project.clone())
}

/// Update the last-seen timestamp for the current project
pub fn update_project_last_seen() -> Result<()> {
    let current_dir = env::current_dir()?;
    let mut registry = ProjectRegistry::load()?;
    registry.update_last_seen(&current_dir)?;
    registry.save()?;
    Ok(())
}
