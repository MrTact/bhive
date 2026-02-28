//! PostgreSQL LISTEN/NOTIFY support for push notifications
//!
//! This module provides real-time event notifications using PostgreSQL's
//! LISTEN/NOTIFY mechanism, allowing clients to receive updates without polling.

use crate::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgListener, PgPool};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Notification channel names
pub mod channels {
    pub const TASK_EVENTS: &str = "task_events";
    pub const OPERATOR_EVENTS: &str = "operator_events";
    pub const ALL_EVENTS: &str = "all_events";
}

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoordinationEvent {
    /// Task was created
    TaskCreated {
        task_id: Uuid,
        description: String,
    },
    /// Task was claimed by an operator
    TaskClaimed {
        task_id: Uuid,
        operator_id: Uuid,
    },
    /// Task was started
    TaskStarted {
        task_id: Uuid,
    },
    /// Task was completed successfully
    TaskCompleted {
        task_id: Uuid,
        result: Option<serde_json::Value>,
    },
    /// Task failed with error
    TaskFailed {
        task_id: Uuid,
        error: String,
    },
    /// Operator was acquired
    OperatorAcquired {
        operator_id: Uuid,
        operator_type: String,
        reused: bool,
    },
    /// Operator was released
    OperatorReleased {
        operator_id: Uuid,
        success: bool,
    },
    /// Operator status changed
    OperatorStatusChanged {
        operator_id: Uuid,
        old_status: String,
        new_status: String,
    },
}

/// Notification listener manages PostgreSQL LISTEN connections
pub struct NotificationListener {
    listener: PgListener,
    sender: broadcast::Sender<CoordinationEvent>,
}

impl NotificationListener {
    /// Create a new notification listener
    pub async fn new(pool: &PgPool) -> Result<Self> {
        let listener = PgListener::connect_with(pool).await?;
        let (sender, _receiver) = broadcast::channel(100);

        Ok(Self { listener, sender })
    }

    /// Subscribe to notifications
    pub fn subscribe(&self) -> broadcast::Receiver<CoordinationEvent> {
        self.sender.subscribe()
    }

    /// Start listening on specific channels
    pub async fn listen(&mut self, channels: &[&str]) -> Result<()> {
        for channel in channels {
            self.listener.listen(channel).await?;
        }
        Ok(())
    }

    /// Start the event loop (run this in a background task)
    pub async fn run(mut self) -> Result<()> {
        loop {
            let notification = self.listener.recv().await?;
            let channel = notification.channel();
            let payload = notification.payload();

            // Parse and broadcast the event
            if let Ok(event) = serde_json::from_str::<CoordinationEvent>(payload) {
                tracing::debug!("Received notification on {}: {:?}", channel, event);
                let _ = self.sender.send(event);
            } else {
                tracing::warn!(
                    "Failed to parse notification on channel {}: {}",
                    channel,
                    payload
                );
            }
        }
    }
}

/// Helper to emit a notification
pub async fn notify(pool: &PgPool, channel: &str, event: &CoordinationEvent) -> Result<()> {
    let payload = serde_json::to_string(event)?;
    sqlx::query("SELECT pg_notify($1, $2)")
        .bind(channel)
        .bind(&payload)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = CoordinationEvent::TaskCreated {
            task_id: Uuid::new_v4(),
            description: "Test task".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: CoordinationEvent = serde_json::from_str(&json).unwrap();

        match parsed {
            CoordinationEvent::TaskCreated { description, .. } => {
                assert_eq!(description, "Test task");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
