use clap::Parser;
use thiserror::Error;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use yinmo_bot::core::daemon::{Daemon, DaemonConfig, SchedulerConfig};
use yinmo_bot::core::gateway::{GatewayAction, GatewayError, GatewayRequest};
use yinmo_bot::core::task::TaskEvent;

#[derive(Debug, Error)]
enum MainError {
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error("failed to listen for shutdown signal: {0}")]
    Signal(std::io::Error),
}

#[derive(Debug, Parser)]
#[command(author, version, about = "yinmo-bot daemon")]
struct Cli {
    #[arg(long, default_value_t = 5)]
    workers: usize,
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    init_tracing();
    let cli = Cli::parse();

    let daemon = Daemon::start(DaemonConfig {
        scheduler: SchedulerConfig {
            worker_limit: cli.workers,
        },
    });

    let gateway = daemon.gateway();
    let mut events = gateway.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                TaskEvent::Queued { task_id } => info!(task_id = %task_id.0, "task queued"),
                TaskEvent::Started { task_id, worker_id } => {
                    info!(task_id = %task_id.0, worker_id, "task started")
                }
                TaskEvent::Finished {
                    task_id,
                    worker_id,
                    result,
                } => match result {
                    Ok(execution) => {
                        info!(
                            task_id = %task_id.0,
                            worker_id,
                            exit_code = execution.exit_code,
                            duration_ms = execution.duration_ms(),
                            "task finished"
                        );
                    }
                    Err(err) => {
                        error!(task_id = %task_id.0, worker_id, error = %err, "task failed")
                    }
                },
            }
        }
    });

    info!("daemon started; press ctrl+c to stop");

    gateway
        .submit(GatewayRequest {
            source: "bootstrap".to_string(),
            action: GatewayAction::RunCommand {
                command: "echo".to_string(),
                args: vec!["yinmo-bot started".to_string()],
                cwd: None,
                env: Default::default(),
            },
            context: Default::default(),
        })
        .await?;

    tokio::signal::ctrl_c().await.map_err(MainError::Signal)?;
    info!("shutdown signal received");
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .compact()
        .init();
}
