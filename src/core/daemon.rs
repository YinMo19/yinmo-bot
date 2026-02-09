use std::sync::Arc;

use crate::core::gateway::{ExternalGateway, UnifiedGateway};
use crate::core::scheduler::Scheduler;
pub use crate::core::scheduler::SchedulerConfig;
use crate::executor::process::ProcessExecutor;

pub const MAX_WORKERS: usize = 5;
pub const MIN_WORKERS: usize = 1;

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub scheduler: SchedulerConfig,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            scheduler: SchedulerConfig::default(),
        }
    }
}

pub struct Daemon {
    gateway: Arc<dyn ExternalGateway>,
}

impl Daemon {
    pub fn start(config: DaemonConfig) -> Self {
        let worker_limit = config
            .scheduler
            .worker_limit
            .clamp(MIN_WORKERS, MAX_WORKERS);
        let scheduler = Scheduler::start(SchedulerConfig { worker_limit }, ProcessExecutor);
        let gateway: Arc<dyn ExternalGateway> = Arc::new(UnifiedGateway::new(scheduler));

        Self { gateway }
    }

    pub fn gateway(&self) -> Arc<dyn ExternalGateway> {
        self.gateway.clone()
    }
}
