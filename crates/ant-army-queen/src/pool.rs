//! Ant pool management

use ant_army_core::coordination::{Ant, AntType};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

/// Information about an ant in the pool
#[derive(Debug, Clone)]
pub struct AntInfo {
    /// The ant record from database
    pub ant: Ant,

    /// Current assigned task (if any)
    pub current_task_id: Option<Uuid>,

    /// Workspace directory for this ant
    pub workspace_path: PathBuf,

    /// When the ant was last active
    pub last_active: Instant,

    /// Process ID of worker (if spawned)
    pub process_id: Option<u32>,
}

/// Ant pool state management
#[derive(Debug, Default)]
pub struct AntPool {
    /// Active ants currently working on tasks
    active: HashMap<Uuid, AntInfo>,

    /// Idle ants available for work
    idle: HashMap<Uuid, AntInfo>,

    /// Task assignments (task_id -> ant_id)
    assignments: HashMap<Uuid, Uuid>,
}

impl AntPool {
    /// Create a new empty ant pool
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an ant to the idle pool
    pub fn add_idle(&mut self, ant: Ant, workspace_path: PathBuf) {
        let info = AntInfo {
            ant,
            current_task_id: None,
            workspace_path,
            last_active: Instant::now(),
            process_id: None,
        };
        self.idle.insert(info.ant.id, info);
    }

    /// Move an ant from idle to active
    pub fn activate(&mut self, ant_id: Uuid, task_id: Uuid, process_id: Option<u32>) -> bool {
        if let Some(mut info) = self.idle.remove(&ant_id) {
            info.current_task_id = Some(task_id);
            info.last_active = Instant::now();
            info.process_id = process_id;
            self.assignments.insert(task_id, ant_id);
            self.active.insert(ant_id, info);
            true
        } else {
            false
        }
    }

    /// Move an ant from active to idle
    pub fn deactivate(&mut self, ant_id: Uuid) -> bool {
        if let Some(mut info) = self.active.remove(&ant_id) {
            if let Some(task_id) = info.current_task_id {
                self.assignments.remove(&task_id);
            }
            info.current_task_id = None;
            info.last_active = Instant::now();
            info.process_id = None;
            self.idle.insert(ant_id, info);
            true
        } else {
            false
        }
    }

    /// Remove an ant from the pool entirely
    pub fn remove(&mut self, ant_id: Uuid) -> Option<AntInfo> {
        if let Some(info) = self.active.remove(&ant_id) {
            if let Some(task_id) = info.current_task_id {
                self.assignments.remove(&task_id);
            }
            Some(info)
        } else {
            self.idle.remove(&ant_id)
        }
    }

    /// Get an idle ant of a specific type
    pub fn get_idle_ant(&self, ant_type: AntType) -> Option<&AntInfo> {
        self.idle
            .values()
            .find(|info| info.ant.ant_type == ant_type)
    }

    /// Get any idle ant (for reuse)
    pub fn get_any_idle_ant(&self) -> Option<&AntInfo> {
        self.idle.values().next()
    }

    /// Get active ant info
    pub fn get_active(&self, ant_id: Uuid) -> Option<&AntInfo> {
        self.active.get(&ant_id)
    }

    /// Get idle ant info
    pub fn get_idle(&self, ant_id: Uuid) -> Option<&AntInfo> {
        self.idle.get(&ant_id)
    }

    /// Get ant assigned to a task
    pub fn get_ant_for_task(&self, task_id: Uuid) -> Option<Uuid> {
        self.assignments.get(&task_id).copied()
    }

    /// Count active ants
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Count idle ants
    pub fn idle_count(&self) -> usize {
        self.idle.len()
    }

    /// Total ants in pool
    pub fn total_count(&self) -> usize {
        self.active.len() + self.idle.len()
    }

    /// Get all idle ants that have been idle longer than duration
    pub fn get_stale_idle_ants(&self, max_age: std::time::Duration) -> Vec<Uuid> {
        let now = Instant::now();
        self.idle
            .iter()
            .filter(|(_, info)| now.duration_since(info.last_active) > max_age)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all idle ant IDs
    pub fn idle_ant_ids(&self) -> HashSet<Uuid> {
        self.idle.keys().copied().collect()
    }

    /// Get all active ant IDs
    pub fn active_ant_ids(&self) -> HashSet<Uuid> {
        self.active.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ant_army_core::coordination::AntStatus;
    use chrono::Utc;
    use std::path::Path;

    fn create_test_ant(ant_type: AntType) -> Ant {
        Ant {
            id: Uuid::new_v4(),
            ant_type,
            status: AntStatus::Idle,
            workspace_path: None,
            current_task_id: None,
            current_session_id: None,
            tasks_completed: 0,
            last_active_at: Some(Utc::now()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_ant_pool_lifecycle() {
        let mut pool = AntPool::new();
        let ant = create_test_ant(AntType::Operator);
        let ant_id = ant.id;
        let task_id = Uuid::new_v4();

        // Add idle ant
        pool.add_idle(ant, PathBuf::from("/tmp/workspace"));
        assert_eq!(pool.idle_count(), 1);
        assert_eq!(pool.active_count(), 0);

        // Activate ant
        assert!(pool.activate(ant_id, task_id, Some(12345)));
        assert_eq!(pool.idle_count(), 0);
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.get_ant_for_task(task_id), Some(ant_id));

        // Deactivate ant
        assert!(pool.deactivate(ant_id));
        assert_eq!(pool.idle_count(), 1);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.get_ant_for_task(task_id), None);

        // Remove ant
        assert!(pool.remove(ant_id).is_some());
        assert_eq!(pool.total_count(), 0);
    }

    #[test]
    fn test_get_idle_ant_by_type() {
        let mut pool = AntPool::new();

        let operator = create_test_ant(AntType::Operator);
        let builder = create_test_ant(AntType::Builder);

        pool.add_idle(operator.clone(), PathBuf::from("/tmp/op"));
        pool.add_idle(builder.clone(), PathBuf::from("/tmp/build"));

        assert!(pool.get_idle_ant(AntType::Operator).is_some());
        assert!(pool.get_idle_ant(AntType::Builder).is_some());
        assert!(pool.get_idle_ant(AntType::Tester).is_none());
    }
}
