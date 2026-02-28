//! Operator pool management

use bhive_core::coordination::{Operator, OperatorType};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

/// Information about an operator in the pool
#[derive(Debug, Clone)]
pub struct OperatorInfo {
    /// The operator record from database
    pub operator: Operator,

    /// Current assigned task (if any)
    pub current_task_id: Option<Uuid>,

    /// Workspace directory for this operator
    pub workspace_path: PathBuf,

    /// When the operator was last active
    pub last_active: Instant,

    /// Tokio task handle ID (for tracking/cancellation)
    pub task_handle_id: Option<u64>,
}

/// Operator pool state management
#[derive(Debug, Default)]
pub struct OperatorPool {
    /// Active operators currently working on tasks
    active: HashMap<Uuid, OperatorInfo>,

    /// Idle operators available for work
    idle: HashMap<Uuid, OperatorInfo>,

    /// Task assignments (task_id -> operator_id)
    assignments: HashMap<Uuid, Uuid>,
}

impl OperatorPool {
    /// Create a new empty operator pool
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an operator to the idle pool
    pub fn add_idle(&mut self, operator: Operator, workspace_path: PathBuf) {
        let info = OperatorInfo {
            operator,
            current_task_id: None,
            workspace_path,
            last_active: Instant::now(),
            task_handle_id: None,
        };
        self.idle.insert(info.operator.id, info);
    }

    /// Move an operator from idle to active
    pub fn activate(&mut self, operator_id: Uuid, task_id: Uuid, task_handle_id: Option<u64>) -> bool {
        if let Some(mut info) = self.idle.remove(&operator_id) {
            info.current_task_id = Some(task_id);
            info.last_active = Instant::now();
            info.task_handle_id = task_handle_id;
            self.assignments.insert(task_id, operator_id);
            self.active.insert(operator_id, info);
            true
        } else {
            false
        }
    }

    /// Move an operator from active to idle
    pub fn deactivate(&mut self, operator_id: Uuid) -> bool {
        if let Some(mut info) = self.active.remove(&operator_id) {
            if let Some(task_id) = info.current_task_id {
                self.assignments.remove(&task_id);
            }
            info.current_task_id = None;
            info.last_active = Instant::now();
            info.task_handle_id = None;
            self.idle.insert(operator_id, info);
            true
        } else {
            false
        }
    }

    /// Remove an operator from the pool entirely
    pub fn remove(&mut self, operator_id: Uuid) -> Option<OperatorInfo> {
        if let Some(info) = self.active.remove(&operator_id) {
            if let Some(task_id) = info.current_task_id {
                self.assignments.remove(&task_id);
            }
            Some(info)
        } else {
            self.idle.remove(&operator_id)
        }
    }

    /// Get an idle operator of a specific type
    pub fn get_idle_operator(&self, operator_type: OperatorType) -> Option<&OperatorInfo> {
        self.idle
            .values()
            .find(|info| info.operator.operator_type == operator_type)
    }

    /// Get any idle operator (for reuse)
    pub fn get_any_idle_operator(&self) -> Option<&OperatorInfo> {
        self.idle.values().next()
    }

    /// Get active operator info
    pub fn get_active(&self, operator_id: Uuid) -> Option<&OperatorInfo> {
        self.active.get(&operator_id)
    }

    /// Get idle operator info
    pub fn get_idle(&self, operator_id: Uuid) -> Option<&OperatorInfo> {
        self.idle.get(&operator_id)
    }

    /// Get operator assigned to a task
    pub fn get_operator_for_task(&self, task_id: Uuid) -> Option<Uuid> {
        self.assignments.get(&task_id).copied()
    }

    /// Count active operators
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Count idle operators
    pub fn idle_count(&self) -> usize {
        self.idle.len()
    }

    /// Total operators in pool
    pub fn total_count(&self) -> usize {
        self.active.len() + self.idle.len()
    }

    /// Get all idle operators that have been idle longer than duration
    pub fn get_stale_idle_operators(&self, max_age: std::time::Duration) -> Vec<Uuid> {
        let now = Instant::now();
        self.idle
            .iter()
            .filter(|(_, info)| now.duration_since(info.last_active) > max_age)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all idle operator IDs
    pub fn idle_operator_ids(&self) -> HashSet<Uuid> {
        self.idle.keys().copied().collect()
    }

    /// Get all active operator IDs
    pub fn active_operator_ids(&self) -> HashSet<Uuid> {
        self.active.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bhive_core::coordination::OperatorStatus;
    use chrono::Utc;
    use std::path::Path;

    fn create_test_operator(operator_type: OperatorType) -> Operator {
        Operator {
            id: Uuid::new_v4(),
            operator_type,
            status: OperatorStatus::Idle,
            workspace_path: None,
            current_task_id: None,
            current_session_id: None,
            tasks_completed: 0,
            last_active_at: Some(Utc::now()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_operator_pool_lifecycle() {
        let mut pool = OperatorPool::new();
        let operator = create_test_operator(OperatorType::Operator);
        let operator_id = operator.id;
        let task_id = Uuid::new_v4();

        // Add idle operator
        pool.add_idle(operator, PathBuf::from("/tmp/workspace"));
        assert_eq!(pool.idle_count(), 1);
        assert_eq!(pool.active_count(), 0);

        // Activate operator
        assert!(pool.activate(operator_id, task_id, Some(12345)));
        assert_eq!(pool.idle_count(), 0);
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.get_operator_for_task(task_id), Some(operator_id));

        // Deactivate operator
        assert!(pool.deactivate(operator_id));
        assert_eq!(pool.idle_count(), 1);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.get_operator_for_task(task_id), None);

        // Remove operator
        assert!(pool.remove(operator_id).is_some());
        assert_eq!(pool.total_count(), 0);
    }

    #[test]
    fn test_get_idle_operator_by_type() {
        let mut pool = OperatorPool::new();

        let operator = create_test_operator(OperatorType::Operator);
        let builder = create_test_operator(OperatorType::Builder);

        pool.add_idle(operator.clone(), PathBuf::from("/tmp/op"));
        pool.add_idle(builder.clone(), PathBuf::from("/tmp/build"));

        assert!(pool.get_idle_operator(OperatorType::Operator).is_some());
        assert!(pool.get_idle_operator(OperatorType::Builder).is_some());
        assert!(pool.get_idle_operator(OperatorType::Tester).is_none());
    }
}
