use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;

use crate::core::scheduler::SchedulerHandle;
use crate::core::task::{Task, TaskEvent, TaskId, TaskSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    pub source: String,
    pub action: GatewayAction,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayAction {
    RunCommand {
        command: String,
        args: Vec<String>,
        cwd: Option<PathBuf>,
        env: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayAck {
    pub task_id: TaskId,
    pub accepted_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("failed to dispatch task: {message}")]
    Dispatch { message: String },
}

#[async_trait]
pub trait ExternalGateway: Send + Sync {
    async fn submit(&self, request: GatewayRequest) -> Result<GatewayAck, GatewayError>;
    fn subscribe_events(&self) -> broadcast::Receiver<TaskEvent>;
}

#[derive(Clone)]
pub struct UnifiedGateway {
    scheduler: SchedulerHandle,
}

impl UnifiedGateway {
    pub fn new(scheduler: SchedulerHandle) -> Self {
        Self { scheduler }
    }
}

#[async_trait]
impl ExternalGateway for UnifiedGateway {
    async fn submit(&self, request: GatewayRequest) -> Result<GatewayAck, GatewayError> {
        let task_spec = match request.action {
            GatewayAction::RunCommand {
                command,
                args,
                cwd,
                env,
            } => TaskSpec {
                requester: request.source,
                command,
                args,
                cwd,
                env,
            },
        };

        let task = Task::new(task_spec);
        let task_id = task.id.clone();
        self.scheduler
            .enqueue(task)
            .await
            .map_err(|error| GatewayError::Dispatch {
                message: error.to_string(),
            })?;

        Ok(GatewayAck {
            task_id,
            accepted_at: Utc::now(),
        })
    }

    fn subscribe_events(&self) -> broadcast::Receiver<TaskEvent> {
        self.scheduler.subscribe_events()
    }
}
