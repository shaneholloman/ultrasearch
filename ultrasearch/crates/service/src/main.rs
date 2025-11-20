//! Entry point for the UltraSearch Windows service (bootstrap only for now).

use anyhow::Result;
use core_types::config::load_config;

mod logging;
mod metrics;

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let cfg = load_config(None)?;
    logging::init(&cfg.logging, &cfg.app.data_dir, "service")?;

    rt.block_on(async move {
        let _metrics = metrics::spawn_basic_metrics(cfg.metrics.sample_interval_secs);
        println!("UltraSearch service placeholder – wiring pending.");
    });

    Ok(())
}
