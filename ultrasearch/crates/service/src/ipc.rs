#![cfg(target_os = "windows")]

use anyhow::Result;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    let mut len_buf = [0u8; 4];
    // Simple length-prefixed framing
    loop {
        if conn.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        if len == 0 || len > MAX_MESSAGE_BYTES {
            tracing::warn!("invalid frame size {}", len);
            break;
        }
        let mut buf = vec![0u8; len];
        conn.read_exact(&mut buf).await?;
        // For now echo the request id if present; real dispatch will live in service.
        // This keeps the pipeline compile-ready and testable.
        let response = echo_id(&buf);
        conn.write_all(&(response.len() as u32).to_le_bytes()).await?;
        conn.write_all(&response).await?;
    }
    Ok(())
}

fn echo_id(payload: &[u8]) -> Vec<u8> {
    // Best-effort: if payload decodes as a UUID (first 16 bytes), echo it back.
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
        let resp = echo_id(id.as_bytes());
        assert_eq!(resp, id.as_bytes());
    }
}
