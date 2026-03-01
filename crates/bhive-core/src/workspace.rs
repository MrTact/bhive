//! Persistent workspace management for operators
//!
//! Manages jujutsu (jj) workspaces that point to a central repository.
//! Each operator gets a persistent workspace at `{project_root}/workspaces/{operator_id}/`
//! that shares history with the central repo at `{project_root}/repo/`.
//!
//! ## Task Workflow
//!
//! Tasks are tracked via jj bookmarks named with the task ID:
//!
//! 1. **First operator on a task**: Creates new commit off trunk + bookmark
//!    - `jj new trunk()` - create commit
//!    - `jj bookmark create {task_id}` - mark it
//!
//! 2. **Subsequent operators on same task**: Sync to bookmark
//!    - `jj edit {task_id}` - edit the bookmarked commit
//!
//! 3. **Task merged**: Delete bookmark after merge to long-lived branch
//!
//! ## Operator Lifecycle
//!
//! 1. **Creation** (Queen spawn): `ensure_exists()` creates a jj workspace if needed
//! 2. **Task start** (Worker): `prepare_for_task()` syncs to task bookmark (or creates it)
//! 3. **Reap** (Queen reaper): `cleanup_operator()` removes workspace entirely
//! 4. **Task complete** (after merge): `cleanup_task()` removes the bookmark

use crate::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use uuid::Uuid;

/// Maximum retries for jj commands (handles transient lock contention)
const MAX_RETRIES: u32 = 3;
/// Base delay between retries (with jitter)
const RETRY_BASE_DELAY_MS: u64 = 100;

/// Manages persistent jj workspaces for operators
#[derive(Debug, Clone)]
pub struct WorkspaceManager {
    /// Revset to rebase workspace onto for clean state (default: "trunk()")
    pub base_revset: String,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self {
            base_revset: "trunk()".to_string(),
        }
    }
}

impl WorkspaceManager {
    /// Create a new workspace manager with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a custom base revset
    pub fn with_base_revset(base_revset: impl Into<String>) -> Self {
        Self {
            base_revset: base_revset.into(),
        }
    }

    /// Get the path to the central repo
    pub fn repo_path(project_root: &Path) -> PathBuf {
        project_root.join("repo")
    }

    /// Get the path to an operator's workspace
    pub fn workspace_path(project_root: &Path, operator_id: Uuid) -> PathBuf {
        project_root.join("workspaces").join(operator_id.to_string())
    }

    /// Ensure a workspace exists for an operator.
    ///
    /// Creates the jj workspace if it doesn't exist or is corrupted.
    /// Called by Queen when spawning a new operator.
    ///
    /// Returns the workspace path.
    pub async fn ensure_exists(
        &self,
        project_root: &Path,
        operator_id: Uuid,
    ) -> Result<PathBuf> {
        let repo_path = Self::repo_path(project_root);
        let workspace_path = Self::workspace_path(project_root, operator_id);

        // Validate central repo exists
        if !repo_path.exists() {
            return Err(Error::Config(format!(
                "Central repo not found at {:?}",
                repo_path
            )));
        }

        // Check if it's a valid jj repo
        if !self.is_valid_jj_repo(&repo_path).await {
            return Err(Error::Config(format!(
                "Path {:?} is not a valid jujutsu repository",
                repo_path
            )));
        }

        // Ensure workspaces directory exists
        let workspaces_dir = project_root.join("workspaces");
        if !workspaces_dir.exists() {
            tokio::fs::create_dir_all(&workspaces_dir).await.map_err(|e| {
                Error::Config(format!("Failed to create workspaces dir: {}", e))
            })?;
        }

        // Check if workspace already exists and is valid
        if workspace_path.exists() {
            if self.is_valid_workspace(&workspace_path).await {
                tracing::debug!("Workspace {:?} already exists and is valid", workspace_path);
                // Run update-stale just in case
                let _ = self.run_jj_update_stale(&workspace_path).await;
                return Ok(workspace_path);
            } else {
                // Workspace exists but is invalid - recover
                tracing::warn!(
                    "Workspace {:?} exists but is invalid, recovering...",
                    workspace_path
                );
                self.recover_workspace(project_root, operator_id).await?;
            }
        }

        // Create new workspace
        self.create_workspace(&repo_path, &workspace_path, operator_id).await?;

        Ok(workspace_path)
    }

    /// Prepare a workspace for a task.
    ///
    /// If this is the first operator on this task:
    /// - Creates a new commit off trunk: `jj new trunk()`
    /// - Creates a bookmark at that commit: `jj bookmark create {task_id}`
    ///
    /// If subsequent operators are working on this task:
    /// - Syncs to the task bookmark: `jj edit {task_id}`
    ///
    /// Called by Worker at the start of each task.
    pub async fn prepare_for_task(
        &self,
        project_root: &Path,
        operator_id: Uuid,
        task_id: Uuid,
    ) -> Result<()> {
        let workspace_path = Self::workspace_path(project_root, operator_id);
        let task_bookmark = task_id.to_string();

        if !workspace_path.exists() {
            return Err(Error::Config(format!(
                "Workspace {:?} does not exist",
                workspace_path
            )));
        }

        tracing::debug!(
            "Preparing workspace {:?} for task {}",
            workspace_path,
            task_id
        );

        // Handle any stale working copy first
        self.run_jj_update_stale(&workspace_path).await?;

        // Check if bookmark exists for this task
        if self.bookmark_exists(&workspace_path, &task_bookmark).await? {
            // Subsequent operator: sync to existing task bookmark
            tracing::info!(
                "Task {} already has bookmark, syncing workspace via jj edit",
                task_id
            );
            self.run_jj_edit(&workspace_path, &task_bookmark).await?;
        } else {
            // First operator: create new commit off trunk and bookmark it
            tracing::info!(
                "First operator on task {}, creating commit and bookmark",
                task_id
            );
            self.run_jj_new(&workspace_path, &self.base_revset).await?;
            self.run_jj_bookmark_create(&workspace_path, &task_bookmark).await?;
        }

        tracing::info!(
            "Workspace {:?} prepared for task {} (bookmark: {})",
            workspace_path,
            task_id,
            task_bookmark
        );

        Ok(())
    }

    /// Clean up a task's bookmark after it has been merged.
    ///
    /// Called after a task is completed and merged into the long-lived branch.
    pub async fn cleanup_task(
        &self,
        project_root: &Path,
        task_id: Uuid,
    ) -> Result<()> {
        let repo_path = Self::repo_path(project_root);
        let task_bookmark = task_id.to_string();

        tracing::info!("Cleaning up bookmark for completed task {}", task_id);

        // Delete the task bookmark (ignore errors if it doesn't exist)
        let _ = self.run_jj_bookmark_delete(&repo_path, &task_bookmark).await;

        Ok(())
    }

    /// Clean up a workspace when an operator is reaped.
    ///
    /// Removes the workspace from jj and deletes the directory.
    /// Called by Queen reaper when operator exceeds idle timeout.
    pub async fn cleanup(
        &self,
        project_root: &Path,
        operator_id: Uuid,
    ) -> Result<()> {
        let repo_path = Self::repo_path(project_root);
        let workspace_path = Self::workspace_path(project_root, operator_id);

        tracing::info!("Cleaning up workspace {:?}", workspace_path);

        // Forget workspace in jj (ignore errors - workspace might already be gone)
        let _ = self.run_jj_workspace_forget(&repo_path, operator_id).await;

        // Remove workspace directory
        if workspace_path.exists() {
            tokio::fs::remove_dir_all(&workspace_path).await.map_err(|e| {
                Error::Config(format!("Failed to remove workspace dir: {}", e))
            })?;
        }

        Ok(())
    }

    /// Check if a workspace is corrupted and needs recovery.
    ///
    /// Returns true if workspace appears valid, false if corrupted.
    pub async fn is_healthy(&self, project_root: &Path, operator_id: Uuid) -> bool {
        let workspace_path = Self::workspace_path(project_root, operator_id);
        self.is_valid_workspace(&workspace_path).await
    }

    /// Recover a corrupted workspace by removing and recreating it.
    async fn recover_workspace(
        &self,
        project_root: &Path,
        operator_id: Uuid,
    ) -> Result<()> {
        let repo_path = Self::repo_path(project_root);
        let workspace_path = Self::workspace_path(project_root, operator_id);

        tracing::warn!("Recovering corrupted workspace {:?}", workspace_path);

        // 1. Forget workspace in jj (ignore errors)
        let _ = self.run_jj_workspace_forget(&repo_path, operator_id).await;

        // 2. Remove directory
        if workspace_path.exists() {
            tokio::fs::remove_dir_all(&workspace_path).await.map_err(|e| {
                Error::Config(format!("Failed to remove corrupted workspace: {}", e))
            })?;
        }

        // 3. Recreate
        self.create_workspace(&repo_path, &workspace_path, operator_id).await
    }

    /// Create a new jj workspace
    async fn create_workspace(
        &self,
        repo_path: &Path,
        workspace_path: &Path,
        operator_id: Uuid,
    ) -> Result<()> {
        tracing::info!(
            "Creating jj workspace {:?} for operator {}",
            workspace_path,
            operator_id
        );

        // jj workspace add --name {operator_id} -r @ {workspace_path}
        let output = self
            .run_jj_with_retry(
                repo_path,
                &[
                    "workspace",
                    "add",
                    "--name",
                    &operator_id.to_string(),
                    "-r",
                    "@",
                    workspace_path.to_str().ok_or_else(|| {
                        Error::Config("Invalid workspace path".to_string())
                    })?,
                ],
            )
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Config(format!(
                "Failed to create jj workspace: {}",
                stderr
            )));
        }

        // Run update-stale to ensure clean state
        let _ = self.run_jj_update_stale(workspace_path).await;

        Ok(())
    }

    /// Check if a path is a valid jj repository
    async fn is_valid_jj_repo(&self, path: &Path) -> bool {
        // Quick check: .jj directory exists
        if !path.join(".jj").exists() {
            return false;
        }

        // Verify with jj root command
        let result = Command::new("jj")
            .args(["root"])
            .current_dir(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        matches!(result, Ok(status) if status.success())
    }

    /// Check if a workspace path is a valid jj workspace
    async fn is_valid_workspace(&self, workspace_path: &Path) -> bool {
        if !workspace_path.exists() {
            return false;
        }

        // Try jj status to verify workspace is functional
        let result = Command::new("jj")
            .args(["status"])
            .current_dir(workspace_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        matches!(result, Ok(status) if status.success())
    }

    /// Run `jj workspace update-stale`
    async fn run_jj_update_stale(&self, workspace_path: &Path) -> Result<()> {
        let output = self
            .run_jj_with_retry(workspace_path, &["workspace", "update-stale"])
            .await?;

        // update-stale succeeds even if nothing was stale
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Log but don't fail - this is a best-effort operation
            tracing::warn!("jj workspace update-stale warning: {}", stderr);
        }

        Ok(())
    }

    /// Run `jj workspace forget {name}` from the repo directory
    async fn run_jj_workspace_forget(&self, repo_path: &Path, operator_id: Uuid) -> Result<()> {
        let output = self
            .run_jj_with_retry(
                repo_path,
                &["workspace", "forget", &operator_id.to_string()],
            )
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Config(format!(
                "Failed to forget workspace: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Check if a bookmark exists
    async fn bookmark_exists(&self, workspace_path: &Path, bookmark: &str) -> Result<bool> {
        let output = self
            .run_jj_with_retry(workspace_path, &["bookmark", "list", bookmark])
            .await?;

        if !output.status.success() {
            // Command failed - treat as bookmark not found
            return Ok(false);
        }

        // If output is non-empty, bookmark exists
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.trim().is_empty())
    }

    /// Run `jj new {revset}` to create a new commit
    async fn run_jj_new(&self, workspace_path: &Path, revset: &str) -> Result<()> {
        let output = self
            .run_jj_with_retry(workspace_path, &["new", revset])
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Config(format!(
                "Failed to create new commit: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Run `jj bookmark create {name}` to create a bookmark at current commit
    async fn run_jj_bookmark_create(&self, workspace_path: &Path, bookmark: &str) -> Result<()> {
        let output = self
            .run_jj_with_retry(workspace_path, &["bookmark", "create", bookmark])
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Config(format!(
                "Failed to create bookmark: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Run `jj bookmark delete {name}` to delete a bookmark
    async fn run_jj_bookmark_delete(&self, workspace_path: &Path, bookmark: &str) -> Result<()> {
        let output = self
            .run_jj_with_retry(workspace_path, &["bookmark", "delete", bookmark])
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Config(format!(
                "Failed to delete bookmark: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Run `jj edit {revset}` to sync workspace to a commit/bookmark
    async fn run_jj_edit(&self, workspace_path: &Path, revset: &str) -> Result<()> {
        let output = self
            .run_jj_with_retry(workspace_path, &["edit", revset])
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Config(format!(
                "Failed to edit revision: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Run a jj command with retry logic for transient failures
    async fn run_jj_with_retry(
        &self,
        cwd: &Path,
        args: &[&str],
    ) -> Result<std::process::Output> {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                // Add jitter to delay
                let delay = RETRY_BASE_DELAY_MS * (1 << attempt)
                    + rand::random::<u64>() % RETRY_BASE_DELAY_MS;
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                tracing::debug!(
                    "Retrying jj command (attempt {}/{}): {:?}",
                    attempt + 1,
                    MAX_RETRIES,
                    args
                );
            }

            let result = Command::new("jj")
                .args(args)
                .current_dir(cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await;

            match result {
                Ok(output) => {
                    // Check for transient errors that warrant retry
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("lock") || stderr.contains("concurrent") {
                            last_error = Some(format!("Lock contention: {}", stderr));
                            continue;
                        }
                    }
                    return Ok(output);
                }
                Err(e) => {
                    last_error = Some(format!("Command execution failed: {}", e));
                    continue;
                }
            }
        }

        Err(Error::Config(format!(
            "jj command failed after {} retries: {}",
            MAX_RETRIES,
            last_error.unwrap_or_else(|| "unknown error".to_string())
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_paths() {
        let project_root = PathBuf::from("/home/user/myproject");
        let operator_id = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();

        assert_eq!(
            WorkspaceManager::repo_path(&project_root),
            PathBuf::from("/home/user/myproject/repo")
        );
        assert_eq!(
            WorkspaceManager::workspace_path(&project_root, operator_id),
            PathBuf::from("/home/user/myproject/workspaces/12345678-1234-1234-1234-123456789abc")
        );
    }

    #[test]
    fn test_default_base_revset() {
        let manager = WorkspaceManager::new();
        assert_eq!(manager.base_revset, "trunk()");
    }

    #[test]
    fn test_custom_base_revset() {
        let manager = WorkspaceManager::with_base_revset("main");
        assert_eq!(manager.base_revset, "main");
    }
}
