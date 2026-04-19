use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, bail};
use tokio::sync::RwLock;

use crate::ipc;
use crate::protocol::{
    ClientRequest, DEFAULT_COLS, DEFAULT_ROWS, KeySpec, ServerResponse, SessionRegistryEntry, TabSummary,
    now_ms,
};
use crate::registry;
use crate::screen::ScreenBuffer;
use crate::winpty::{PtySession, default_shell};

struct RuntimeState {
    cwd: PathBuf,
    tabs: HashMap<String, Arc<TabState>>,
}

struct TabState {
    name: String,
    shell: String,
    cols: Arc<RwLock<u16>>,
    rows: Arc<RwLock<u16>>,
    last_activity_ms: Arc<RwLock<u128>>,
    created_at_ms: u128,
    screen: Arc<RwLock<ScreenBuffer>>,
    writer: Arc<tokio::sync::Mutex<Box<dyn std::io::Write + Send>>>,
    _session: Arc<PtySession>,
}

pub async fn serve(session_key: String, cwd: PathBuf) -> Result<()> {
    let endpoint = ipc::pipe_name(&session_key);

    if let Some(existing) = registry::read_entry(&session_key)? {
        if existing.pid != std::process::id() && ipc::send_request::<_, ServerResponse>(&existing.endpoint, &ClientRequest::ListTabs).await.is_ok() {
            return Ok(());
        }
    }

    let state = Arc::new(RwLock::new(RuntimeState {
        cwd: cwd.clone(),
        tabs: HashMap::new(),
    }));

    registry::write_entry(&SessionRegistryEntry::new(
        session_key.clone(),
        cwd.display().to_string(),
        endpoint.clone(),
        std::process::id(),
    ))?;

    let accept_state = Arc::clone(&state);
    let accept_key = session_key.clone();
    let accept_result = ipc::accept_loop(&endpoint, move |payload| {
        let state = Arc::clone(&accept_state);
        let accept_key = accept_key.clone();
        async move {
            let request = serde_json::from_slice::<ClientRequest>(&payload)?;
            let response = handle_request(state, request, &accept_key).await.unwrap_or_else(|error| {
                ServerResponse::Error {
                    code: "runtime".to_string(),
                    message: format!("{error:#}"),
                }
            });
            Ok(serde_json::to_vec(&response)?)
        }
    })
    .await;

    let cleanup = registry::delete_entry(&session_key);
    accept_result?;
    cleanup?;
    Ok(())
}

async fn handle_request(
    state: Arc<RwLock<RuntimeState>>,
    request: ClientRequest,
    session_key: &str,
) -> Result<ServerResponse> {
    match request {
        ClientRequest::OpenTab { tab, cols, rows } => {
            ensure_tab(
                &state,
                &tab,
                cols.unwrap_or(DEFAULT_COLS),
                rows.unwrap_or(DEFAULT_ROWS),
            )
            .await?;
            Ok(ServerResponse::Ok)
        }
        ClientRequest::Snapshot { tab, wait_stable_ms } => {
            let tab_state = ensure_tab(&state, &tab, DEFAULT_COLS, DEFAULT_ROWS).await?;
            wait_stable(&tab_state, wait_stable_ms).await;
            let cols = *tab_state.cols.read().await;
            let rows = *tab_state.rows.read().await;
            let text = tab_state.screen.read().await.viewport_text();
            Ok(ServerResponse::SnapshotText { tab, cols, rows, text })
        }
        ClientRequest::ExecLine { tab, line } => {
            let tab_state = ensure_tab(&state, &tab, DEFAULT_COLS, DEFAULT_ROWS).await?;
            write_bytes(&tab_state, line.as_bytes()).await?;
            write_bytes(&tab_state, &KeySpec {
                key: crate::protocol::KeyCodeSpec::Enter,
                ctrl: false,
                alt: false,
                shift: false,
                meta: false,
            }
            .to_bytes()?)
            .await?;
            Ok(ServerResponse::Ok)
        }
        ClientRequest::TypeText { tab, text } => {
            let tab_state = ensure_tab(&state, &tab, DEFAULT_COLS, DEFAULT_ROWS).await?;
            write_bytes(&tab_state, text.as_bytes()).await?;
            Ok(ServerResponse::Ok)
        }
        ClientRequest::PressKey { tab, key } => {
            let tab_state = ensure_tab(&state, &tab, DEFAULT_COLS, DEFAULT_ROWS).await?;
            write_bytes(&tab_state, &key.to_bytes()?).await?;
            Ok(ServerResponse::Ok)
        }
        ClientRequest::MouseEvent { tab, event } => {
            let tab_state = ensure_tab(&state, &tab, DEFAULT_COLS, DEFAULT_ROWS).await?;
            write_bytes(&tab_state, &event.to_escape()).await?;
            Ok(ServerResponse::Ok)
        }
        ClientRequest::ResizeTab { tab, cols, rows } => {
            let tab_state = ensure_tab(&state, &tab, cols, rows).await?;
            tab_state._session.resize(cols, rows)?;
            *tab_state.cols.write().await = cols;
            *tab_state.rows.write().await = rows;
            tab_state.screen.write().await.resize(cols, rows);
            Ok(ServerResponse::Ok)
        }
        ClientRequest::ListTabs => {
            let tabs = {
                let state = state.read().await;
                let mut tabs = Vec::new();
                for tab in state.tabs.values() {
                    tabs.push(TabSummary {
                        name: tab.name.clone(),
                        shell: tab.shell.clone(),
                        cols: *tab.cols.read().await,
                        rows: *tab.rows.read().await,
                        created_at_ms: tab.created_at_ms,
                        last_activity_at_ms: *tab.last_activity_ms.read().await,
                    });
                }
                tabs
            };
            Ok(ServerResponse::TabList { tabs })
        }
        ClientRequest::CloseTab { tab } => {
            let removed = state.write().await.tabs.remove(&tab);
            if removed.is_none() {
                bail!("tab `{tab}` does not exist");
            }
            Ok(ServerResponse::Ok)
        }
        ClientRequest::CloseAll => {
            state.write().await.tabs.clear();
            registry::delete_entry(session_key)?;
            std::process::exit(0);
        }
    }
}

async fn ensure_tab(
    state: &Arc<RwLock<RuntimeState>>,
    tab: &str,
    cols: u16,
    rows: u16,
) -> Result<Arc<TabState>> {
    if let Some(existing) = state.read().await.tabs.get(tab).cloned() {
        return Ok(existing);
    }

    let cwd = state.read().await.cwd.clone();
    let shell = default_shell();
    let session = Arc::new(PtySession::new(&shell, &cwd, cols, rows)?);
    let screen = Arc::new(RwLock::new(ScreenBuffer::new(cols, rows)));
    let created_at = now_ms();
    let tab_state = Arc::new(TabState {
        name: tab.to_string(),
        shell: shell.clone(),
        cols: Arc::new(RwLock::new(cols)),
        rows: Arc::new(RwLock::new(rows)),
        last_activity_ms: Arc::new(RwLock::new(created_at)),
        created_at_ms: created_at,
        screen: Arc::clone(&screen),
        writer: Arc::clone(&session.writer),
        _session: Arc::clone(&session),
    });

    spawn_reader(Arc::clone(&tab_state), session)?;

    let mut state = state.write().await;
    let entry = state
        .tabs
        .entry(tab.to_string())
        .or_insert_with(|| Arc::clone(&tab_state))
        .clone();
    Ok(entry)
}

fn spawn_reader(tab_state: Arc<TabState>, session: Arc<PtySession>) -> Result<()> {
    let screen = Arc::clone(&tab_state.screen);
    let activity = Arc::clone(&tab_state.last_activity_ms);
    let runtime_handle = tokio::runtime::Handle::current();
    let reader = Arc::clone(&session.reader);
    let writer = Arc::clone(&tab_state.writer);

    std::thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            let read_result = {
                let mut reader = reader.blocking_lock();
                reader.read(&mut buffer)
            };
            match read_result {
                Ok(0) => break,
                Ok(bytes_read) => {
                    let owned = buffer[..bytes_read].to_vec();
                    if owned.as_slice() == b"\x1b[6n" {
                        let writer = Arc::clone(&writer);
                        runtime_handle.block_on(async move {
                            let mut writer = writer.lock().await;
                            let _ = writer.write_all(b"\x1b[1;1R");
                            let _ = writer.flush();
                        });
                        continue;
                    }
                    let screen = Arc::clone(&screen);
                    let activity = Arc::clone(&activity);
                    runtime_handle.block_on(async move {
                        screen.write().await.apply(&owned);
                        *activity.write().await = now_ms();
                    });
                }
                Err(_) => break,
            }
        }
    });

    Ok(())
}

async fn write_bytes(tab: &TabState, bytes: &[u8]) -> Result<()> {
    let mut writer = tab.writer.lock().await;
    use std::io::Write;
    writer.write_all(bytes)?;
    writer.flush()?;
    *tab.last_activity_ms.write().await = now_ms();
    Ok(())
}

async fn wait_stable(tab: &TabState, wait_stable_ms: u64) {
    let interval = Duration::from_millis(wait_stable_ms.max(1));
    loop {
        let before = *tab.last_activity_ms.read().await;
        tokio::time::sleep(interval).await;
        let after = *tab.last_activity_ms.read().await;
        if before == after {
            break;
        }
    }
}
