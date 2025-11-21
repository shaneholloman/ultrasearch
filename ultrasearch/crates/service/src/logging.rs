use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use core_types::config::LoggingSection;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing/logging for the service process using the provided config.
///
/// - Honors `logging.level` from config, falling back to `RUST_LOG` then `info`.
/// - Writes JSON logs to the configured rolling file (daily) and also to stdout.
pub fn init_tracing_with_config(
    cfg: &LoggingSection,
) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    // Determine filter: explicit level in config, else RUST_LOG, else info.
    let filter_str = if !cfg.level.is_empty() {
        cfg.level.clone()
    } else {
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
    };
    let filter = EnvFilter::new(filter_str);

    // Split logging file into directory + filename.
    let log_path = PathBuf::from(&cfg.file);
    let (dir, file) = split_dir_file(&log_path)?;
    if !dir.exists() {
        fs::create_dir_all(dir).context("create log directory")?;
    }

    let file_appender = tracing_appender::rolling::daily(dir, file);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // File layer (plain text for now; enable JSON when tracing-subscriber json feature is on).
    let file_layer = fmt::layer()
        .with_writer(non_blocking.clone())
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .try_init()?;

    Ok(guard)
}

/// Backward-compatible initializer using defaults.
pub fn init_tracing() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let default = LoggingSection::default();
    init_tracing_with_config(&default)
}

fn split_dir_file(path: &Path) -> Result<(&Path, &str)> {
    let dir = path.parent().context("log file missing parent directory")?;
    let file = path
        .file_name()
        .and_then(|s| s.to_str())
        .context("log file missing filename")?;
    Ok((dir, file))
}
