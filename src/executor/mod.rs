pub mod process;

use async_trait::async_trait;
use thiserror::Error;

use crate::core::task::{Task, TaskExecution};
use crate::executor::process::ProcessExecutorError;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error(transparent)]
    Process(#[from] ProcessExecutorError),
}

#[async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, task: Task) -> Result<TaskExecution, ExecutorError>;
}
