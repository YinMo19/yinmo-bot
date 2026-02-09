use std::process::Stdio;

use async_trait::async_trait;
use chrono::Utc;
use thiserror::Error;
use tokio::process::Command;

use crate::core::task::{Task, TaskExecution};
use crate::executor::{ExecutorError, TaskExecutor};

#[derive(Debug, Clone, Default)]
pub struct ProcessExecutor;

#[derive(Debug, Error)]
pub enum ProcessExecutorError {
    #[error("failed to execute command `{command}`: {source}")]
    SpawnOutput {
        command: String,
        #[source]
        source: std::io::Error,
    },
}

#[async_trait]
impl TaskExecutor for ProcessExecutor {
    async fn execute(&self, task: Task) -> Result<TaskExecution, ExecutorError> {
        let mut command = Command::new(&task.spec.command);
        command
            .args(&task.spec.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(cwd) = &task.spec.cwd {
            command.current_dir(cwd);
        }

        if !task.spec.env.is_empty() {
            command.envs(task.spec.env.iter());
        }

        let started_at = Utc::now();
        let output =
            command
                .output()
                .await
                .map_err(|source| ProcessExecutorError::SpawnOutput {
                    command: task.spec.command.clone(),
                    source,
                })?;
        let finished_at = Utc::now();

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(TaskExecution {
            exit_code,
            stdout,
            stderr,
            started_at,
            finished_at,
        })
    }
}
