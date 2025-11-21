//! Service support library: tracing/logging bootstrap and metrics helpers.

mod logging;
pub mod metrics;
pub mod status;
pub mod priority;

pub use logging::init as init_tracing;
pub use metrics::{
    init_metrics_from_config, scrape_metrics, ServiceMetrics, ServiceMetricsSnapshot,
};
pub use priority::{set_process_priority, ProcessPriority};
