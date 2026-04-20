use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "tuiless")]
#[command(about = "Stateless CLI driving a stateful per-workspace terminal runtime")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(hide = true)]
    Serve {
        #[arg(long)]
        session_key: String,
        #[arg(long)]
        cwd: PathBuf,
    },
    Open(OpenArgs),
    Snapshot(SnapshotArgs),
    Fetch(SnapshotArgs),
    Exec(TabLineArgs),
    Type(TabLineArgs),
    Press(PressArgs),
    Click(ClickArgs),
    Drag(DragArgs),
    Wheel(WheelArgs),
    MouseDown(MousePointArgs),
    MouseUp(MousePointArgs),
    MouseMove(MouseMoveArgs),
    Resize(ResizeArgs),
    Attach(AttachArgs),
    List,
    Close(CloseArgs),
}

#[derive(Args, Debug)]
pub struct OpenArgs {
    pub tab: String,
    #[arg(long)]
    pub cols: Option<u16>,
    #[arg(long)]
    pub rows: Option<u16>,
}

#[derive(Args, Debug)]
pub struct SnapshotArgs {
    pub tab: String,
    #[arg(long = "wait-stable", default_value_t = crate::protocol::DEFAULT_WAIT_STABLE_MS)]
    pub wait_stable_ms: u64,
}

#[derive(Args, Debug)]
pub struct TabLineArgs {
    pub tab: String,
    pub line: String,
}

#[derive(Args, Debug)]
pub struct PressArgs {
    pub tab: String,
    pub key: String,
    #[arg(long)]
    pub ctrl: bool,
    #[arg(long)]
    pub alt: bool,
    #[arg(long)]
    pub shift: bool,
    #[arg(long)]
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MouseButtonCli {
    Left,
    Right,
    Middle,
}

#[derive(Args, Debug)]
pub struct ClickArgs {
    pub tab: String,
    #[arg(long)]
    pub x: u16,
    #[arg(long)]
    pub y: u16,
    #[arg(long, value_enum, default_value = "left")]
    pub button: MouseButtonCli,
}

#[derive(Args, Debug)]
pub struct DragArgs {
    pub tab: String,
    #[arg(long = "from-x")]
    pub from_x: u16,
    #[arg(long = "from-y")]
    pub from_y: u16,
    #[arg(long = "to-x")]
    pub to_x: u16,
    #[arg(long = "to-y")]
    pub to_y: u16,
    #[arg(long, value_enum, default_value = "left")]
    pub button: MouseButtonCli,
}

#[derive(Args, Debug)]
pub struct WheelArgs {
    pub tab: String,
    #[arg(long = "delta-y", allow_negative_numbers = true)]
    pub delta_y: i16,
    #[arg(long)]
    pub x: Option<u16>,
    #[arg(long)]
    pub y: Option<u16>,
}

#[derive(Args, Debug)]
pub struct MousePointArgs {
    pub tab: String,
    #[arg(long)]
    pub x: u16,
    #[arg(long)]
    pub y: u16,
    #[arg(long, value_enum, default_value = "left")]
    pub button: MouseButtonCli,
}

#[derive(Args, Debug)]
pub struct MouseMoveArgs {
    pub tab: String,
    #[arg(long)]
    pub x: u16,
    #[arg(long)]
    pub y: u16,
}

#[derive(Args, Debug)]
pub struct ResizeArgs {
    pub tab: String,
    #[arg(long)]
    pub cols: u16,
    #[arg(long)]
    pub rows: u16,
}

#[derive(Args, Debug)]
pub struct AttachArgs {
    pub tab: String,
    #[arg(long = "wait-stable", default_value_t = crate::protocol::DEFAULT_WAIT_STABLE_MS)]
    pub wait_stable_ms: u64,
}

#[derive(Args, Debug)]
pub struct CloseArgs {
    pub tab: Option<String>,
    #[arg(long)]
    pub all: bool,
}

impl Cli {
    pub fn parse_from_env() -> Result<Self> {
        Ok(Self::parse())
    }
}
