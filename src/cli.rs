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
    Fetch(FetchArgs),
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
    #[arg(long)]
    pub color: Option<SnapshotColorCli>,
    #[arg(long, requires = "color", value_parser = parse_snapshot_theme)]
    pub theme: Option<crate::protocol::SnapshotTheme>,
}

#[derive(Args, Debug)]
pub struct FetchArgs {
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

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SnapshotColorCli {
    Smart,
    Foreground,
    Background,
    #[value(name = "foreground,background")]
    ForegroundBackground,
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

impl From<SnapshotColorCli> for crate::protocol::SnapshotColorMode {
    fn from(value: SnapshotColorCli) -> Self {
        match value {
            SnapshotColorCli::Smart => Self::Smart,
            SnapshotColorCli::Foreground => Self::Foreground,
            SnapshotColorCli::Background => Self::Background,
            SnapshotColorCli::ForegroundBackground => Self::ForegroundBackground,
        }
    }
}

fn parse_snapshot_theme(
    input: &str,
) -> std::result::Result<crate::protocol::SnapshotTheme, String> {
    crate::protocol::SnapshotTheme::parse_cli_name(input).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, SnapshotColorCli};
    use crate::protocol::SnapshotTheme;

    #[test]
    fn snapshot_color_modes_parse() {
        let smart = Cli::try_parse_from(["tuiless", "snapshot", "demo", "--color", "smart"])
            .expect("smart should parse");
        let fg = Cli::try_parse_from(["tuiless", "snapshot", "demo", "--color", "foreground"])
            .expect("foreground should parse");
        let bg = Cli::try_parse_from(["tuiless", "snapshot", "demo", "--color", "background"])
            .expect("background should parse");
        let both = Cli::try_parse_from([
            "tuiless",
            "snapshot",
            "demo",
            "--color",
            "foreground,background",
        ])
        .expect("foreground,background should parse");

        let Command::Snapshot(smart_args) = smart.command else {
            panic!("expected snapshot command");
        };
        assert!(matches!(smart_args.color, Some(SnapshotColorCli::Smart)));
        assert_eq!(smart_args.theme, None);

        let Command::Snapshot(fg_args) = fg.command else {
            panic!("expected snapshot command");
        };
        assert!(matches!(fg_args.color, Some(SnapshotColorCli::Foreground)));

        let Command::Snapshot(bg_args) = bg.command else {
            panic!("expected snapshot command");
        };
        assert!(matches!(bg_args.color, Some(SnapshotColorCli::Background)));

        let Command::Snapshot(both_args) = both.command else {
            panic!("expected snapshot command");
        };
        assert!(matches!(
            both_args.color,
            Some(SnapshotColorCli::ForegroundBackground)
        ));
    }

    #[test]
    fn snapshot_invalid_color_or_empty_value_errors() {
        let invalid = Cli::try_parse_from(["tuiless", "snapshot", "demo", "--color", "invalid"]);
        assert!(invalid.is_err());

        let empty = Cli::try_parse_from(["tuiless", "snapshot", "demo", "--color", ""]);
        assert!(empty.is_err());
    }

    #[test]
    fn snapshot_repeated_color_is_invalid() {
        let repeated = Cli::try_parse_from([
            "tuiless",
            "snapshot",
            "demo",
            "--color",
            "smart",
            "--color",
            "foreground",
        ]);
        assert!(repeated.is_err());
    }

    #[test]
    fn snapshot_theme_defaults_and_parses() {
        let with_theme = Cli::try_parse_from([
            "tuiless",
            "snapshot",
            "demo",
            "--color",
            "foreground",
            "--theme",
            "One Half Dark",
        ])
        .expect("theme should parse");
        let Command::Snapshot(args) = with_theme.command else {
            panic!("expected snapshot command");
        };
        assert_eq!(args.theme, Some(SnapshotTheme::OneHalfDark));

        let invalid = Cli::try_parse_from([
            "tuiless",
            "snapshot",
            "demo",
            "--color",
            "foreground",
            "--theme",
            "Unknown Theme",
        ]);
        assert!(invalid.is_err());
    }

    #[test]
    fn fetch_keeps_snapshot_wait_stable_shape_without_color_flags() {
        let parsed =
            Cli::try_parse_from(["tuiless", "fetch", "demo", "--wait-stable", "220"]).unwrap();
        let Command::Fetch(args) = parsed.command else {
            panic!("expected fetch command");
        };
        assert_eq!(args.tab, "demo");
        assert_eq!(args.wait_stable_ms, 220);
    }
}
