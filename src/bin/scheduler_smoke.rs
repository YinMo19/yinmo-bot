use std::collections::HashSet;

use thiserror::Error;
use yinmo_bot::core::daemon::{Daemon, DaemonConfig, SchedulerConfig};
use yinmo_bot::core::gateway::{GatewayAction, GatewayError, GatewayRequest};
use yinmo_bot::core::task::TaskEvent;

#[derive(Debug, Error)]
enum SmokeError {
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error("max concurrency exceeded limit: observed {observed}, limit {limit}")]
    ConcurrencyExceeded { observed: usize, limit: usize },
}

#[tokio::main]
async fn main() -> Result<(), SmokeError> {
    let daemon = Daemon::start(DaemonConfig {
        scheduler: SchedulerConfig { worker_limit: 5 },
    });
    let gateway = daemon.gateway();

    let mut events = gateway.subscribe_events();
    let mut finished = HashSet::new();
    let mut inflight = 0usize;
    let mut max_inflight = 0usize;

    for _ in 0..20 {
        gateway
            .submit(GatewayRequest {
                source: "smoke".to_string(),
                action: GatewayAction::RunCommand {
                    command: "sh".to_string(),
                    args: vec!["-c".to_string(), "sleep 0.15".to_string()],
                    cwd: None,
                    env: Default::default(),
                },
                context: Default::default(),
            })
            .await?;
    }

    while finished.len() < 20 {
        if let Ok(event) = events.recv().await {
            match event {
                TaskEvent::Started { .. } => {
                    inflight += 1;
                    if inflight > max_inflight {
                        max_inflight = inflight;
                    }
                }
                TaskEvent::Finished { task_id, .. } => {
                    inflight = inflight.saturating_sub(1);
                    finished.insert(task_id.0);
                }
                TaskEvent::Queued { .. } => {}
            }
        }
    }

    if max_inflight > 5 {
        return Err(SmokeError::ConcurrencyExceeded {
            observed: max_inflight,
            limit: 5,
        });
    }

    println!("scheduler smoke passed; observed max concurrency: {max_inflight}");
    Ok(())
}
