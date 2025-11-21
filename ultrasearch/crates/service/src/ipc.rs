#![cfg(target_os = "windows")]

use anyhow::Result;
use ipc::{framing, SearchRequest, StatusRequest, StatusResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::task::JoinHandle;
use uuid::Uuid;

const DEFAULT_PIPE_NAME: &str = r#"\\.\pipe\ultrasearch"#;
const MAX_MESSAGE_BYTES: usize = 256 * 1024;

/// Start a Tokio named-pipe server that spawns a task per connection.
pub async fn start_pipe_server(pipe_name: Option<&str>) -> Result<JoinHandle<()>> {
    let name = pipe_name.unwrap_or(DEFAULT_PIPE_NAME);
    let server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(name)?;

    let handle = tokio::spawn(async move {
        loop {
            let mut conn = server
                .connect()
                .await
                .expect("named pipe connect failed");

            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut conn).await {
                    tracing::warn!("pipe connection error: {e:?}");
                }
            });
        }
    });

    Ok(handle)
}

async fn handle_connection(conn: &mut NamedPipeServer) -> Result<()> {
    loop {
        // decode frame
        let mut len_prefix = [0u8; 4];
        if conn.read_exact(&mut len_prefix).await.is_err() {
            break;
        }
        let frame_len = u32::from_le_bytes(len_prefix) as usize;
        if frame_len == 0 || frame_len > MAX_MESSAGE_BYTES {
            tracing::warn!("invalid frame size {}", frame_len);
            break;
        }
        let mut buf = vec![0u8; frame_len];
        conn.read_exact(&mut buf).await?;

        let payload = match framing::decode_frame(&buf) {
            Ok((payload, _rem)) => payload,
            Err(e) => {
                tracing::warn!("failed to decode frame: {e}");
                continue;
            }
        };

        let response = dispatch(&payload);
        let framed = framing::encode_frame(&response).unwrap_or_default();
        conn.write_all(&(framed.len() as u32).to_le_bytes()).await?;
        conn.write_all(&framed).await?;
    }
    Ok(())
}

fn dispatch(payload: &[u8]) -> Vec<u8> {
    // Try StatusRequest first.
    if let Ok(req) = bincode::deserialize::<StatusRequest>(payload) {
        let resp = StatusResponse {
            id: req.id,
            volumes: Vec::new(),
            last_index_commit_ts: None,
            scheduler_state: "unknown".into(),
            metrics: None,
        };
        return bincode::serialize(&resp).unwrap_or_default();
    }
    // Fallback: echo SearchRequest id.
    if let Ok(req) = bincode::deserialize::<SearchRequest>(payload) {
        return bincode::serialize(&req.id).unwrap_or_default();
    }
    // If payload decodes as a UUID prefix, echo it back.
    if payload.len() >= 16 {
        if let Ok(id) = Uuid::from_slice(&payload[..16]) {
            return id.as_bytes().to_vec();
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn echoes_uuid_prefix() {
        let id = Uuid::new_v4();
        let resp = dispatch(id.as_bytes());
        assert_eq!(resp, id.as_bytes());
    }

    #[test]
    fn status_request_roundtrip() {
        let req = StatusRequest { id: Uuid::new_v4() };
        let resp_bytes = dispatch(&bincode::serialize(&req).unwrap());
        let resp: StatusResponse = bincode::deserialize(&resp_bytes).unwrap();
        assert_eq!(resp.id, req.id);
        assert!(resp.volumes.is_empty());
    }

    #[test]
    fn search_request_echoes_id() {
        let req = SearchRequest {
            id: Uuid::new_v4(),
            query: ipc::QueryExpr::Term(ipc::TermExpr {
                field: None,
                value: "x".into(),
                modifier: ipc::TermModifier::Term,
            }),
            limit: 1,
            mode: ipc::SearchMode::Auto,
            timeout: None,
            offset: 0,
        };
        let resp_bytes = dispatch(&bincode::serialize(&req).unwrap());
        let echoed: Uuid = bincode::deserialize(&resp_bytes).unwrap();
        assert_eq!(echoed, req.id);
    }
}
