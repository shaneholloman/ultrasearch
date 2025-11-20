//! Service support library: tracing/logging bootstrap and future host wiring.

mod logging;

pub use logging::init_tracing;

/// Placeholder service entry; will be wired to windows-services host.
pub fn start_service() {
    // TODO(c00.2.4): integrate windows-services crate and Tokio runtime.
    // TODO(c00.8.2): add file logging sink (per-process path under %PROGRAMDATA%/UltraSearch/log).
    if let Err(e) = init_tracing() {
        eprintln!("Failed to init tracing: {e}");
    }
}

/// Initialize tracing/logging for the service process.
///
/// - Reads filter from `RUST_LOG` (defaults to `info`).
/// - Formats logs with timestamps, level, and thread id.
/// - Designed to be called early in service/worker startup.
pub fn init_tracing() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(false)
        .with_line_number(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .map_err(Into::into)
}

/// Placeholder service entry; will be wired to windows-services host.
pub fn start_service() {
    // TODO(c00.2.4): integrate windows-services crate and Tokio runtime.
    // TODO(c00.8.2): add file logging sink (per-process path under %PROGRAMDATA%/UltraSearch/log).
    if let Err(e) = init_tracing() {
        eprintln!("Failed to init tracing: {e}");
    }
}
