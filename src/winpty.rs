use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use tokio::sync::Mutex;

pub struct PtySession {
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub reader: Arc<Mutex<Box<dyn Read + Send>>>,
    pub shell: String,
    master: Mutex<Box<dyn MasterPty + Send>>,
}

impl PtySession {
    pub fn new(shell: &str, cwd: &Path, cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut command = shell_command(shell);
        command.cwd(cwd.as_os_str());
        let _child = pair.slave.spawn_command(command)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
            shell: shell.to_string(),
            master: Mutex::new(pair.master),
        })
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.master.blocking_lock().resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }
}

pub fn default_shell() -> String {
    let candidates = ["cmd.exe", "pwsh.exe", "powershell.exe"];
    for candidate in candidates {
        if shell_exists(candidate) {
            if candidate.eq_ignore_ascii_case("cmd.exe") {
                return "cmd.exe".to_string();
            }
            return candidate.to_string();
        }
    }
    "cmd.exe".to_string()
}

fn shell_exists(name: &str) -> bool {
    std::process::Command::new("where")
        .arg(name)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn shell_command(shell: &str) -> CommandBuilder {
    if shell.eq_ignore_ascii_case("cmd.exe") {
        let mut cmd = CommandBuilder::new("cmd.exe");
        cmd.args(["/Q", "/K"]);
        cmd
    } else {
        CommandBuilder::new(shell)
    }
}
