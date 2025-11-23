use anyhow::Result;
use core_types::config::AppConfig;
use ipc::VolumeStatus;
use ntfs_watcher::{NtfsError, discover_volumes, enumerate_mft};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::status_provider::{update_status_last_commit, update_status_volumes};
use crate::meta_ingest::ingest_with_paths;

pub fn scan_volumes(cfg: &AppConfig) -> Result<()> {
    tracing::info!("Starting volume scan...");
    let all_volumes = match discover_volumes() {
        Ok(v) if v.is_empty() => {
            tracing::info!("no NTFS volumes discovered");
            return Ok(());
        }
        Ok(v) => {
            tracing::info!("Discovered {} NTFS volumes total.", v.len());
            v
        }
        Err(NtfsError::NotSupported) => {
            tracing::info!("platform does not support NTFS watcher");
            return Ok(());
        }
        Err(err) => {
            tracing::warn!(error = %err, "failed to discover volumes");
            return Ok(());
        }
    };

    // Filter based on config
    let volumes: Vec<_> = if cfg.volumes.is_empty() {
        // If no volumes specified, maybe default to all? 
        // Or if onboarding wizard sets it, we respect it.
        // "First-Run" usually sets it.
        // If empty, we scan nothing? Or all?
        // Let's default to ALL if empty for backward compat/simplicity, 
        // OR assume empty means "configured to scan nothing".
        // The Wizard sets "Select Drives" (defaults to all fixed).
        // So if config is empty, it might mean "not set up yet".
        // But `bootstrap.rs` used to scan all.
        // Let's keep "Scan All" if config.volumes is empty to avoid breaking existing behavior.
        all_volumes
    } else {
        all_volumes.into_iter().filter(|v| {
            // Check if any drive letter matches config
            // Config has "C:\" or "D:\"
            // VolumeInfo has `drive_letters` vec['C']
            v.drive_letters.iter().any(|l| {
                let mount = format!("{}:\\", l);
                cfg.volumes.contains(&mount)
            })
        }).collect()
    };

    if volumes.is_empty() {
        tracing::info!("No volumes matched configuration.");
        return Ok(());
    }

    let mut status = Vec::with_capacity(volumes.len());

    for volume in volumes {
        tracing::info!(guid = %volume.guid_path, letters = ?volume.drive_letters, "enumerating MFT for volume");
        match enumerate_mft(&volume) {
            Ok(metas) => {
                if metas.is_empty() {
                    tracing::info!(guid = %volume.guid_path, "no entries found during MFT enumeration");
                    continue;
                }

                let count = metas.len() as u64;
                tracing::info!(guid = %volume.guid_path, files = count, "ingesting metadata batch into meta-index");
                match ingest_with_paths(&cfg.paths, metas, None) {
                    Ok(_) => tracing::info!("Successfully ingested {} files.", count),
                    Err(e) => tracing::error!("Failed to ingest files: {}", e),
                }

                status.push(VolumeStatus {
                    volume: volume.id,
                    indexed_files: count,
                    pending_files: 0,
                    last_usn: None,
                    journal_id: None,
                });

                update_status_last_commit(Some(unix_timestamp_secs()));
            }
            Err(err) => {
                let msg = err.to_string();
                if msg.contains("Access is denied") || msg.contains("privilege") {
                    tracing::error!(
                        guid = %volume.guid_path,
                        "CRITICAL: Failed to enumerate MFT due to permissions. Please run the application as Administrator."
                    );
                } else {
                    tracing::warn!(
                        guid = %volume.guid_path,
                        error = %err,
                        "failed to enumerate MFT; skipping volume"
                    );
                }
            }
        }
    }

    if !status.is_empty() {
        update_status_volumes(status);
    }

    Ok(())
}

fn unix_timestamp_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
