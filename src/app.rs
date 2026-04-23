use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::attach;
use crate::cli;
use crate::ipc;
use crate::protocol::{
    ClientRequest, KeySpec, ModifierFlags, MouseButtonSpec, MouseEventSpec, ServerResponse,
    SnapshotColorRequest, SnapshotRenderMode, SnapshotTheme, parse_key_spec,
};
use crate::registry;
use crate::session::{canonical_session_key, normalize_cwd};

pub struct RuntimeClient {
    endpoint: String,
}

const SKILL_NAME: &str = "tuiless";
const SKILL_FILES: &[(&str, &str)] = &[
    ("SKILL.md", include_str!("../skills/tuiless/SKILL.md")),
    (
        "agents/openai.yaml",
        include_str!("../skills/tuiless/agents/openai.yaml"),
    ),
    (
        "references/basic-usage.md",
        include_str!("../skills/tuiless/references/basic-usage.md"),
    ),
    (
        "references/detailed-documentation.md",
        include_str!("../skills/tuiless/references/detailed-documentation.md"),
    ),
    (
        "references/best-practices.md",
        include_str!("../skills/tuiless/references/best-practices.md"),
    ),
];

impl RuntimeClient {
    async fn send(&self, request: &ClientRequest) -> Result<ServerResponse> {
        ipc::send_request::<_, ServerResponse>(&self.endpoint, request).await
    }

    pub async fn open(&self, tab: &str, cols: Option<u16>, rows: Option<u16>) -> Result<()> {
        expect_ok(
            self.send(&ClientRequest::OpenTab {
                tab: tab.to_string(),
                cols,
                rows,
            })
            .await?,
        )
    }

    pub async fn snapshot_raw(
        &self,
        tab: &str,
        wait_stable_ms: u64,
        color: Option<SnapshotColorRequest>,
        render: SnapshotRenderMode,
    ) -> Result<ServerResponse> {
        self.send(&ClientRequest::Snapshot {
            tab: tab.to_string(),
            wait_stable_ms,
            color,
            render,
        })
        .await
    }

    pub async fn fetch_raw(&self, tab: &str, wait_stable_ms: u64) -> Result<ServerResponse> {
        self.send(&ClientRequest::Fetch {
            tab: tab.to_string(),
            wait_stable_ms,
        })
        .await
    }

    pub async fn snapshot_text(
        &self,
        tab: &str,
        wait_stable_ms: u64,
        color: Option<SnapshotColorRequest>,
    ) -> Result<String> {
        match self
            .snapshot_raw(tab, wait_stable_ms, color, SnapshotRenderMode::PlainText)
            .await?
        {
            ServerResponse::SnapshotText { text, .. } => Ok(text),
            other => bail!("unexpected snapshot response: {other:?}"),
        }
    }

    pub async fn snapshot_ansi_text(&self, tab: &str, wait_stable_ms: u64) -> Result<String> {
        match self
            .snapshot_raw(tab, wait_stable_ms, None, SnapshotRenderMode::Ansi)
            .await?
        {
            ServerResponse::SnapshotText { text, .. } => Ok(text),
            other => bail!("unexpected snapshot response: {other:?}"),
        }
    }

    pub async fn fetch_text(&self, tab: &str, wait_stable_ms: u64) -> Result<String> {
        match self.fetch_raw(tab, wait_stable_ms).await? {
            ServerResponse::FetchText { text, .. } => Ok(text),
            other => bail!("unexpected fetch response: {other:?}"),
        }
    }

    pub async fn exec(&self, tab: &str, line: &str) -> Result<()> {
        expect_ok(
            self.send(&ClientRequest::ExecLine {
                tab: tab.to_string(),
                line: line.to_string(),
            })
            .await?,
        )
    }

    pub async fn type_text(&self, tab: &str, text: &str) -> Result<()> {
        expect_ok(
            self.send(&ClientRequest::TypeText {
                tab: tab.to_string(),
                text: text.to_string(),
            })
            .await?,
        )
    }

    pub async fn press(&self, tab: &str, key: KeySpec) -> Result<()> {
        expect_ok(
            self.send(&ClientRequest::PressKey {
                tab: tab.to_string(),
                key,
            })
            .await?,
        )
    }

    pub async fn mouse_event(&self, tab: &str, event: MouseEventSpec) -> Result<()> {
        expect_ok(
            self.send(&ClientRequest::MouseEvent {
                tab: tab.to_string(),
                event,
            })
            .await?,
        )
    }

    pub async fn resize(&self, tab: &str, cols: u16, rows: u16) -> Result<()> {
        expect_ok(
            self.send(&ClientRequest::ResizeTab {
                tab: tab.to_string(),
                cols,
                rows,
            })
            .await?,
        )
    }

    pub async fn list(&self) -> Result<Vec<crate::protocol::TabSummary>> {
        match self.send(&ClientRequest::ListTabs).await? {
            ServerResponse::TabList { tabs } => Ok(tabs),
            other => bail!("unexpected list response: {other:?}"),
        }
    }

    pub async fn close_tab(&self, tab: &str) -> Result<()> {
        expect_ok(
            self.send(&ClientRequest::CloseTab {
                tab: tab.to_string(),
            })
            .await?,
        )
    }

    pub async fn close_all(&self) -> Result<()> {
        expect_ok(self.send(&ClientRequest::CloseAll).await?)
    }
}

pub async fn run(command: cli::Command, cwd: PathBuf) -> Result<()> {
    let cwd = normalize_cwd(&cwd)?;

    if let cli::Command::Skill(args) = &command {
        let installed_path = install_skill_folder(&args.path)?;
        println!("{}", installed_path.display());
        return Ok(());
    }

    let session_key = canonical_session_key(&cwd)?;
    let client = ensure_runtime(&cwd, &session_key).await?;

    match command {
        cli::Command::Open(args) => {
            client.open(&args.tab, args.cols, args.rows).await?;
        }
        cli::Command::Snapshot(args) => {
            let color = args.color.map(|mode| SnapshotColorRequest {
                mode: mode.into(),
                theme: args.theme.unwrap_or_else(SnapshotTheme::default_theme),
            });
            let text = client
                .snapshot_text(&args.tab, args.wait_stable_ms, color)
                .await?;
            print!("{text}");
        }
        cli::Command::Fetch(args) => {
            let text = client.fetch_text(&args.tab, args.wait_stable_ms).await?;
            print!("{text}");
        }
        cli::Command::Exec(args) => {
            client.exec(&args.tab, &args.line).await?;
        }
        cli::Command::Type(args) => {
            client.type_text(&args.tab, &args.line).await?;
        }
        cli::Command::Press(args) => {
            let key = parse_key_spec(
                &args.key,
                &ModifierFlags {
                    ctrl: args.ctrl,
                    alt: args.alt,
                    shift: args.shift,
                    meta: args.meta,
                },
            )?;
            client.press(&args.tab, key).await?;
        }
        cli::Command::Click(args) => {
            let button = mouse_button(args.button);
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Down {
                        x: args.x,
                        y: args.y,
                        button,
                    },
                )
                .await?;
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Up {
                        x: args.x,
                        y: args.y,
                        button,
                    },
                )
                .await?;
        }
        cli::Command::Drag(args) => {
            let button = mouse_button(args.button);
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Down {
                        x: args.from_x,
                        y: args.from_y,
                        button,
                    },
                )
                .await?;
            for point in interpolate_points(args.from_x, args.from_y, args.to_x, args.to_y) {
                client
                    .mouse_event(
                        &args.tab,
                        MouseEventSpec::Move {
                            x: point.0,
                            y: point.1,
                        },
                    )
                    .await?;
            }
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Up {
                        x: args.to_x,
                        y: args.to_y,
                        button,
                    },
                )
                .await?;
        }
        cli::Command::Wheel(args) => {
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Wheel {
                        x: args.x,
                        y: args.y,
                        delta_y: args.delta_y,
                    },
                )
                .await?;
        }
        cli::Command::MouseDown(args) => {
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Down {
                        x: args.x,
                        y: args.y,
                        button: mouse_button(args.button),
                    },
                )
                .await?;
        }
        cli::Command::MouseUp(args) => {
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Up {
                        x: args.x,
                        y: args.y,
                        button: mouse_button(args.button),
                    },
                )
                .await?;
        }
        cli::Command::MouseMove(args) => {
            client
                .mouse_event(
                    &args.tab,
                    MouseEventSpec::Move {
                        x: args.x,
                        y: args.y,
                    },
                )
                .await?;
        }
        cli::Command::Resize(args) => {
            client.resize(&args.tab, args.cols, args.rows).await?;
        }
        cli::Command::Attach(args) => {
            attach::attach(&client, &args.tab, args.wait_stable_ms).await?;
        }
        cli::Command::List => {
            for tab in client.list().await? {
                println!(
                    "{}\t{}\t{}x{}\tcreated={}\tlast_activity={}",
                    tab.name,
                    tab.shell,
                    tab.cols,
                    tab.rows,
                    tab.created_at_ms,
                    tab.last_activity_at_ms
                );
            }
        }
        cli::Command::Close(args) => {
            if args.all {
                client.close_all().await?;
            } else if let Some(tab) = args.tab {
                client.close_tab(&tab).await?;
            } else {
                bail!("close requires either a tab name or --all");
            }
        }
        cli::Command::Skill { .. } => unreachable!("skill is handled before runtime setup"),
        cli::Command::Serve { .. } => unreachable!("serve is handled before app::run"),
    }

    Ok(())
}

async fn ensure_runtime(cwd: &PathBuf, session_key: &str) -> Result<RuntimeClient> {
    if let Some(entry) = registry::read_entry(session_key)? {
        if let Ok(client) = try_connect(entry.endpoint.clone()).await {
            return Ok(client);
        }
        if !process_exists(entry.pid) {
            let _ = registry::delete_entry(session_key);
        } else {
            bail!(
                "runtime process {} is alive but endpoint {} is not connectable",
                entry.pid,
                entry.endpoint
            );
        }
    }

    spawn_runtime(cwd, session_key)?;

    let endpoint = ipc::pipe_name(session_key);
    for _ in 0..100 {
        if let Some(entry) = registry::read_entry(session_key)?
            && let Ok(client) = try_connect(entry.endpoint.clone()).await
        {
            return Ok(client);
        }
        if let Ok(client) = try_connect(endpoint.clone()).await {
            return Ok(client);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    bail!("timed out waiting for runtime for session `{session_key}`");
}

async fn try_connect(endpoint: String) -> Result<RuntimeClient> {
    let client = RuntimeClient { endpoint };
    match client.send(&ClientRequest::ListTabs).await {
        Ok(ServerResponse::TabList { .. }) => Ok(client),
        Ok(ServerResponse::Error { code, message }) => bail!("{code}: {message}"),
        Ok(other) => bail!("unexpected runtime response: {other:?}"),
        Err(error) => Err(error),
    }
}

fn spawn_runtime(cwd: &PathBuf, session_key: &str) -> Result<()> {
    if registry::read_entry(session_key)?.is_some() {
        return Ok(());
    }

    let exe = std::env::current_exe().context("failed to locate current executable")?;
    ProcessCommand::new(exe)
        .arg("serve")
        .arg("--session-key")
        .arg(session_key)
        .arg("--cwd")
        .arg(cwd)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_remove("Path")
        .env_remove("PATH")
        .spawn()
        .context("failed to spawn background runtime")?;
    Ok(())
}

fn process_exists(pid: u32) -> bool {
    std::process::Command::new("cmd")
        .args(["/c", "tasklist", "/FI", &format!("PID eq {pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|output| {
            let text = String::from_utf8_lossy(&output.stdout);
            text.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

fn mouse_button(button: cli::MouseButtonCli) -> MouseButtonSpec {
    match button {
        cli::MouseButtonCli::Left => MouseButtonSpec::Left,
        cli::MouseButtonCli::Right => MouseButtonSpec::Right,
        cli::MouseButtonCli::Middle => MouseButtonSpec::Middle,
    }
}

fn expect_ok(response: ServerResponse) -> Result<()> {
    match response {
        ServerResponse::Ok => Ok(()),
        ServerResponse::Error { code, message } => bail!("{code}: {message}"),
        other => bail!("unexpected runtime response: {other:?}"),
    }
}

fn install_skill_folder(destination_root: &Path) -> Result<PathBuf> {
    if destination_root.exists() && !destination_root.is_dir() {
        bail!(
            "skill destination root is not a directory: {}",
            destination_root.display()
        );
    }
    std::fs::create_dir_all(destination_root).with_context(|| {
        format!(
            "failed to create skill destination root {}",
            destination_root.display()
        )
    })?;

    let skill_dir = destination_root.join(SKILL_NAME);
    if skill_dir.exists() {
        bail!(
            "skill destination already exists: {}",
            skill_dir.as_path().display()
        );
    }

    for (relative_path, content) in SKILL_FILES {
        let output_path = skill_dir.join(relative_path);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create directory {}",
                    parent.as_os_str().to_string_lossy()
                )
            })?;
        }
        std::fs::write(&output_path, content)
            .with_context(|| format!("failed to write skill file {}", output_path.display()))?;
    }

    Ok(skill_dir)
}

fn interpolate_points(from_x: u16, from_y: u16, to_x: u16, to_y: u16) -> Vec<(u16, u16)> {
    let steps = u16::max(from_x.abs_diff(to_x), from_y.abs_diff(to_y)).max(1);
    (1..steps)
        .map(|step| {
            let x = from_x as i32 + ((to_x as i32 - from_x as i32) * step as i32) / steps as i32;
            let y = from_y as i32 + ((to_y as i32 - from_y as i32) * step as i32) / steps as i32;
            (x as u16, y as u16)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{SKILL_FILES, SKILL_NAME, install_skill_folder};

    fn unique_temp_dir(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "tuiless-skill-install-{suffix}-{}-{nanos}",
            std::process::id()
        ))
    }

    #[test]
    fn install_skill_folder_copies_all_embedded_files() {
        let root = unique_temp_dir("copy");
        let skill_dir = install_skill_folder(&root).expect("skill install should succeed");
        assert_eq!(skill_dir, root.join(SKILL_NAME));

        for (relative_path, content) in SKILL_FILES {
            let written = std::fs::read_to_string(skill_dir.join(relative_path))
                .expect("skill file should be written");
            assert_eq!(written, *content);
        }

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn install_skill_folder_errors_when_destination_exists() {
        let root = unique_temp_dir("exists");
        install_skill_folder(&root).expect("first install should succeed");
        let second = install_skill_folder(&root);
        assert!(second.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
