//! Entry point for the UltraSearch Windows service (bootstrap only for now).

use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use anyhow::Result;
use clap::Parser;
use core_types::config::load_or_create_config;
use scheduler::SchedulerConfig;
use service::{
    init_tracing_with_config,
    metrics::{init_metrics_from_config, set_global_metrics, spawn_metrics_http},
    scheduler_runtime::SchedulerRuntime,
};

#[derive(Parser, Debug)]
#[command(name = "searchd", about = "UltraSearch service host")]
struct Args {
    /// Path to config TOML (created if missing)
    #[arg(long, default_value = "config/config.toml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let cfg = load_or_create_config(Some(&args.config))?;
    let _guard = init_tracing_with_config(&cfg.logging)?;

    // Install status provider so IPC/status can respond.
    service::status_provider::init_basic_status_provider();

    if cfg.metrics.enabled {
        let metrics = Arc::new(init_metrics_from_config(&cfg.metrics)?);
        set_global_metrics(metrics);
        spawn_metrics_http(&cfg.metrics.bind)?;
    }

    // Background scheduler sampling loop; real queues/workers will hook in later.
    let sched_cfg = SchedulerConfig {
        warm_idle: Duration::from_secs(cfg.scheduler.idle_warm_seconds),
        deep_idle: Duration::from_secs(cfg.scheduler.idle_deep_seconds),
        cpu_metadata_max: cfg.scheduler.cpu_soft_limit_pct as f32,
        cpu_content_max: cfg.scheduler.cpu_hard_limit_pct as f32,
        disk_busy_threshold_bps: cfg.scheduler.disk_busy_bytes_per_s,
        content_batch_size: cfg.scheduler.content_batch_size as usize,
        ..SchedulerConfig::default()
    };
    let sample_every = Duration::from_secs(cfg.metrics.sample_interval_secs.max(1));
    thread::spawn(move || {
        let mut runtime = SchedulerRuntime::new(sched_cfg);
        loop {
            let _ = runtime.tick();
            thread::sleep(sample_every);
        }
    });

    println!(
        "UltraSearch service placeholder – scheduler sampling active (config: {}).",
        args.config.display()
    );

    Ok(())
}
