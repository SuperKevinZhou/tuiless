use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, PipeMode, ServerOptions};
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

pub struct HandlerResponse {
    pub bytes: Vec<u8>,
    pub shutdown: bool,
}

impl HandlerResponse {
    pub fn continue_with(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            shutdown: false,
        }
    }

    pub fn shutdown_with(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            shutdown: true,
        }
    }
}

pub fn pipe_name(session_key: &str) -> String {
    format!(r"\\.\pipe\tuiless-{}", session_key)
}

pub async fn send_request<T, U>(endpoint: &str, message: &T) -> Result<U>
where
    T: Serialize,
    U: DeserializeOwned,
{
    let mut client = None;
    let mut last_error = None;
    for _ in 0..20 {
        match ClientOptions::new()
            .pipe_mode(PipeMode::Message)
            .open(endpoint)
        {
            Ok(pipe) => {
                client = Some(pipe);
                break;
            }
            Err(error) => {
                last_error = Some(error);
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        }
    }
    let mut client = client
        .ok_or_else(|| {
            last_error.unwrap_or_else(|| std::io::Error::other("failed to open named pipe"))
        })
        .with_context(|| format!("failed to connect to runtime pipe {endpoint}"))?;

    let bytes = serde_json::to_vec(message)?;
    timeout(
        Duration::from_secs(5),
        client.write_u32_le(bytes.len() as u32),
    )
    .await??;
    timeout(Duration::from_secs(5), client.write_all(&bytes)).await??;
    timeout(Duration::from_secs(5), client.flush()).await??;

    let response_len = timeout(Duration::from_secs(5), client.read_u32_le()).await??;
    let mut buffer = vec![0u8; response_len as usize];
    timeout(Duration::from_secs(5), client.read_exact(&mut buffer)).await??;
    let response = serde_json::from_slice(&buffer)?;
    Ok(response)
}

pub async fn accept_loop<F, Fut>(endpoint: &str, handler: F) -> Result<()>
where
    F: FnMut(Vec<u8>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<HandlerResponse>> + Send + 'static,
{
    let mut server = create_server(endpoint, true)?;
    let handler = Arc::new(Mutex::new(handler));

    loop {
        server.connect().await?;
        let connected = server;
        server = create_server(endpoint, false)?;
        if handle_client(connected, Arc::clone(&handler)).await? {
            break;
        }
    }

    Ok(())
}

fn create_server(
    endpoint: &str,
    first_pipe_instance: bool,
) -> Result<tokio::net::windows::named_pipe::NamedPipeServer> {
    ServerOptions::new()
        .pipe_mode(PipeMode::Message)
        .first_pipe_instance(first_pipe_instance)
        .create(endpoint)
        .with_context(|| format!("failed to create pipe server {endpoint}"))
}

async fn handle_client<F, Fut>(
    mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    handler: Arc<Mutex<F>>,
) -> Result<bool>
where
    F: FnMut(Vec<u8>) -> Fut,
    Fut: std::future::Future<Output = Result<HandlerResponse>>,
{
    let request_len = pipe.read_u32_le().await?;
    let mut buffer = vec![0u8; request_len as usize];
    pipe.read_exact(&mut buffer).await?;
    let response = {
        let mut handler = handler.lock().await;
        handler(buffer).await
    }
    .unwrap_or_else(|error| {
        let bytes = serde_json::to_vec(&crate::protocol::ServerResponse::Error {
            code: "internal".to_string(),
            message: format!("{error:#}"),
        })
        .unwrap_or_else(|_| Vec::new());
        HandlerResponse::continue_with(bytes)
    });
    if response.bytes.is_empty() {
        return Err(anyhow!("response serialization failed"));
    }
    pipe.write_u32_le(response.bytes.len() as u32).await?;
    pipe.write_all(&response.bytes).await?;
    pipe.flush().await?;
    Ok(response.shutdown)
}
