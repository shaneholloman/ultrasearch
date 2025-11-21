use ipc::{MetricsSnapshot, StatusResponse, VolumeStatus};
use std::time::SystemTime;

/// Build a StatusResponse from provided fragments.
///
/// This keeps server wiring centralized and ensures new fields are populated consistently.
pub fn make_status_response(
    id: uuid::Uuid,
    volumes: Vec<VolumeStatus>,
    scheduler_state: String,
    metrics: Option<MetricsSnapshot>,
    last_index_commit_ts: Option<i64>,
) -> StatusResponse {
    StatusResponse {
        id,
        volumes,
        scheduler_state,
        last_index_commit_ts: last_index_commit_ts.or_else(now_ts),
        metrics,
    }
}

fn now_ts() -> Option<i64> {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn populates_defaults() {
        let resp = make_status_response(
            Uuid::nil(),
            vec![],
            "idle".into(),
            None,
            None,
        );
        assert!(resp.last_index_commit_ts.is_some());
    }
}
