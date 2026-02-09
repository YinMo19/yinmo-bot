use std::sync::Arc;

use async_channel::{Receiver, Sender};
use thiserror::Error;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::core::task::{Task, TaskEvent, TaskRunError};
use crate::executor::TaskExecutor;

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub worker_limit: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self { worker_limit: 5 }
    }
}

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("task queue is closed")]
    QueueClosed,
}

#[derive(Clone)]
pub struct SchedulerHandle {
    task_tx: Sender<Task>,
    events_tx: broadcast::Sender<TaskEvent>,
}

impl SchedulerHandle {
    pub async fn enqueue(&self, task: Task) -> Result<(), SchedulerError> {
        let task_id = task.id.clone();
        self.events_tx.send(TaskEvent::Queued { task_id }).ok();
        self.task_tx
            .send(task)
            .await
            .map_err(|_| SchedulerError::QueueClosed)?;
        Ok(())
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<TaskEvent> {
        self.events_tx.subscribe()
    }
}

pub struct Scheduler;

impl Scheduler {
    pub fn start<E>(config: SchedulerConfig, executor: E) -> SchedulerHandle
    where
        E: TaskExecutor + 'static,
    {
        let executor = Arc::new(executor);
        let (task_tx, task_rx) = async_channel::unbounded::<Task>();
        let (events_tx, _) = broadcast::channel::<TaskEvent>(256);

        for worker_id in 0..config.worker_limit {
            spawn_worker(
                worker_id,
                task_rx.clone(),
                events_tx.clone(),
                executor.clone(),
            );
        }

        SchedulerHandle { task_tx, events_tx }
    }
}

fn spawn_worker(
    worker_id: usize,
    task_rx: Receiver<Task>,
    events_tx: broadcast::Sender<TaskEvent>,
    executor: Arc<dyn TaskExecutor>,
) {
    tokio::spawn(async move {
        info!(worker_id, "worker started");
        while let Ok(task) = task_rx.recv().await {
            let task_id = task.id.clone();
            events_tx
                .send(TaskEvent::Started {
                    task_id: task_id.clone(),
                    worker_id,
                })
                .ok();

            let result = executor
                .execute(task)
                .await
                .map_err(|err| TaskRunError::Executor {
                    message: err.to_string(),
                });

            if let Err(send_err) = events_tx.send(TaskEvent::Finished {
                task_id,
                worker_id,
                result,
            }) {
                error!(worker_id, error = %send_err, "failed to broadcast task finished event");
            }
        }
        info!(worker_id, "worker exited");
    });
}
