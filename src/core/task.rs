use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub requester: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub created_at: DateTime<Utc>,
    pub spec: TaskSpec,
}

impl Task {
    pub fn new(spec: TaskSpec) -> Self {
        Self {
            id: TaskId::new(),
            created_at: Utc::now(),
            spec,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

impl TaskExecution {
    pub fn duration_ms(&self) -> i64 {
        (self.finished_at - self.started_at).num_milliseconds()
    }
}

#[derive(Debug, Clone, Error)]
pub enum TaskRunError {
    #[error("executor error: {message}")]
    Executor { message: String },
}

#[derive(Debug, Clone)]
pub enum TaskEvent {
    Queued {
        task_id: TaskId,
    },
    Started {
        task_id: TaskId,
        worker_id: usize,
    },
    Finished {
        task_id: TaskId,
        worker_id: usize,
        result: Result<TaskExecution, TaskRunError>,
    },
}
