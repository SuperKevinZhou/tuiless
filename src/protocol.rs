use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow, bail};
use crossterm::event::{KeyCode, MouseButton};
use serde::{Deserialize, Serialize};

pub const DEFAULT_COLS: u16 = 80;
pub const DEFAULT_ROWS: u16 = 24;
pub const DEFAULT_WAIT_STABLE_MS: u64 = 150;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotColorMode {
    Smart,
    Foreground,
    Background,
    ForegroundBackground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotColorLayer {
    Foreground,
    Background,
}

impl SnapshotColorMode {
    pub fn ordered_layers(self) -> &'static [SnapshotColorLayer] {
        match self {
            SnapshotColorMode::Smart => &[SnapshotColorLayer::Foreground],
            SnapshotColorMode::Foreground => &[SnapshotColorLayer::Foreground],
            SnapshotColorMode::Background => &[SnapshotColorLayer::Background],
            SnapshotColorMode::ForegroundBackground => &[
                SnapshotColorLayer::Foreground,
                SnapshotColorLayer::Background,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SnapshotRenderMode {
    #[default]
    PlainText,
    Ansi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotTheme {
    Dimidium,
    Ottosson,
    Campbell,
    CampbellPowershell,
    Vintage,
    OneHalfDark,
    OneHalfLight,
    SolarizedDark,
    SolarizedLight,
    TangoDark,
    TangoLight,
    DarkPlus,
    Cga,
    Ibm5153,
}

impl SnapshotTheme {
    pub fn default_theme() -> Self {
        Self::OneHalfDark
    }

    pub fn parse_cli_name(input: &str) -> Result<Self> {
        let normalized = normalize_theme_name(input);
        let theme = match normalized.as_str() {
            "dimidium" => Self::Dimidium,
            "ottosson" => Self::Ottosson,
            "campbell" => Self::Campbell,
            "campbellpowershell" => Self::CampbellPowershell,
            "vintage" => Self::Vintage,
            "onehalfdark" => Self::OneHalfDark,
            "onehalflight" => Self::OneHalfLight,
            "solarizeddark" => Self::SolarizedDark,
            "solarizedlight" => Self::SolarizedLight,
            "tangodark" => Self::TangoDark,
            "tangolight" => Self::TangoLight,
            "darkplus" => Self::DarkPlus,
            "cga" => Self::Cga,
            "ibm5153" => Self::Ibm5153,
            _ => {
                bail!(
                    "unsupported theme `{input}`; supported themes: {}",
                    Self::all_theme_names().join(", ")
                );
            }
        };
        Ok(theme)
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Dimidium => "Dimidium",
            Self::Ottosson => "Ottosson",
            Self::Campbell => "Campbell",
            Self::CampbellPowershell => "Campbell Powershell",
            Self::Vintage => "Vintage",
            Self::OneHalfDark => "One Half Dark",
            Self::OneHalfLight => "One Half Light",
            Self::SolarizedDark => "Solarized Dark",
            Self::SolarizedLight => "Solarized Light",
            Self::TangoDark => "Tango Dark",
            Self::TangoLight => "Tango Light",
            Self::DarkPlus => "Dark+",
            Self::Cga => "CGA",
            Self::Ibm5153 => "IBM 5153",
        }
    }

    pub fn all_theme_names() -> Vec<&'static str> {
        vec![
            Self::Dimidium.display_name(),
            Self::Ottosson.display_name(),
            Self::Campbell.display_name(),
            Self::CampbellPowershell.display_name(),
            Self::Vintage.display_name(),
            Self::OneHalfDark.display_name(),
            Self::OneHalfLight.display_name(),
            Self::SolarizedDark.display_name(),
            Self::SolarizedLight.display_name(),
            Self::TangoDark.display_name(),
            Self::TangoLight.display_name(),
            Self::DarkPlus.display_name(),
            Self::Cga.display_name(),
            Self::Ibm5153.display_name(),
        ]
    }

    pub fn ansi16_hex(self, slot: u8) -> Option<&'static str> {
        let table = match self {
            Self::Dimidium => &[
                "#000000", "#CF494C", "#60B442", "#DB9C11", "#0575D8", "#AF5ED2", "#1DB6BB",
                "#BAB7B6", "#817E7E", "#FF643B", "#37E57B", "#FCCD1A", "#688DFD", "#ED6FE9",
                "#32E0FB", "#DEE3E4",
            ],
            Self::Ottosson => &[
                "#000000", "#be2c21", "#3fae3a", "#be9a4a", "#204dbe", "#bb54be", "#00a7b2",
                "#bebebe", "#808080", "#ff3e30", "#58ea51", "#ffc944", "#2f6aff", "#fc74ff",
                "#00e1f0", "#ffffff",
            ],
            Self::Campbell => &[
                "#0C0C0C", "#C50F1F", "#13A10E", "#C19C00", "#0037DA", "#881798", "#3A96DD",
                "#CCCCCC", "#767676", "#E74856", "#16C60C", "#F9F1A5", "#3B78FF", "#B4009E",
                "#61D6D6", "#F2F2F2",
            ],
            Self::CampbellPowershell => &[
                "#0C0C0C", "#C50F1F", "#13A10E", "#C19C00", "#0037DA", "#881798", "#3A96DD",
                "#CCCCCC", "#767676", "#E74856", "#16C60C", "#F9F1A5", "#3B78FF", "#B4009E",
                "#61D6D6", "#F2F2F2",
            ],
            Self::Vintage => &[
                "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080",
                "#C0C0C0", "#808080", "#FF0000", "#00FF00", "#FFFF00", "#0000FF", "#FF00FF",
                "#00FFFF", "#FFFFFF",
            ],
            Self::OneHalfDark => &[
                "#282C34", "#E06C75", "#98C379", "#E5C07B", "#61AFEF", "#C678DD", "#56B6C2",
                "#DCDFE4", "#5A6374", "#E06C75", "#98C379", "#E5C07B", "#61AFEF", "#C678DD",
                "#56B6C2", "#DCDFE4",
            ],
            Self::OneHalfLight => &[
                "#383A42", "#E45649", "#50A14F", "#C18301", "#0184BC", "#A626A4", "#0997B3",
                "#FAFAFA", "#4F525D", "#DF6C75", "#98C379", "#E4C07A", "#61AFEF", "#C577DD",
                "#56B5C1", "#FFFFFF",
            ],
            Self::SolarizedDark => &[
                "#002B36", "#DC322F", "#859900", "#B58900", "#268BD2", "#D33682", "#2AA198",
                "#EEE8D5", "#073642", "#CB4B16", "#586E75", "#657B83", "#839496", "#6C71C4",
                "#93A1A1", "#FDF6E3",
            ],
            Self::SolarizedLight => &[
                "#002B36", "#DC322F", "#859900", "#B58900", "#268BD2", "#D33682", "#2AA198",
                "#EEE8D5", "#073642", "#CB4B16", "#586E75", "#657B83", "#839496", "#6C71C4",
                "#93A1A1", "#FDF6E3",
            ],
            Self::TangoDark => &[
                "#000000", "#CC0000", "#4E9A06", "#C4A000", "#3465A4", "#75507B", "#06989A",
                "#D3D7CF", "#555753", "#EF2929", "#8AE234", "#FCE94F", "#729FCF", "#AD7FA8",
                "#34E2E2", "#EEEEEC",
            ],
            Self::TangoLight => &[
                "#000000", "#CC0000", "#4E9A06", "#C4A000", "#3465A4", "#75507B", "#06989A",
                "#D3D7CF", "#555753", "#EF2929", "#8AE234", "#FCE94F", "#729FCF", "#AD7FA8",
                "#34E2E2", "#EEEEEC",
            ],
            Self::DarkPlus => &[
                "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd",
                "#e5e5e5", "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6",
                "#29b8db", "#e5e5e5",
            ],
            Self::Cga => &[
                "#000000", "#AA0000", "#00AA00", "#AA5500", "#0000AA", "#AA00AA", "#00AAAA",
                "#AAAAAA", "#555555", "#FF5555", "#55FF55", "#FFFF55", "#5555FF", "#FF55FF",
                "#55FFFF", "#FFFFFF",
            ],
            Self::Ibm5153 => &[
                "#000000", "#AA0000", "#00AA00", "#C47E00", "#0000AA", "#AA00AA", "#00AAAA",
                "#AAAAAA", "#555555", "#FF5555", "#55FF55", "#FFFF55", "#5555FF", "#FF55FF",
                "#55FFFF", "#FFFFFF",
            ],
        };

        table.get(usize::from(slot)).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotColorRequest {
    pub mode: SnapshotColorMode,
    pub theme: SnapshotTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotColorMetadata {
    pub mode: SnapshotColorMode,
    pub theme: SnapshotTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRegistryEntry {
    pub session_key: String,
    pub cwd: String,
    pub endpoint: String,
    pub pid: u32,
    pub started_at_ms: u128,
}

impl SessionRegistryEntry {
    pub fn new(session_key: String, cwd: String, endpoint: String, pid: u32) -> Self {
        Self {
            session_key,
            cwd,
            endpoint,
            pid,
            started_at_ms: now_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    OpenTab {
        tab: String,
        cols: Option<u16>,
        rows: Option<u16>,
    },
    Snapshot {
        tab: String,
        wait_stable_ms: u64,
        color: Option<SnapshotColorRequest>,
        #[serde(default)]
        render: SnapshotRenderMode,
    },
    Fetch {
        tab: String,
        wait_stable_ms: u64,
    },
    ExecLine {
        tab: String,
        line: String,
    },
    TypeText {
        tab: String,
        text: String,
    },
    PressKey {
        tab: String,
        key: KeySpec,
    },
    MouseEvent {
        tab: String,
        event: MouseEventSpec,
    },
    ResizeTab {
        tab: String,
        cols: u16,
        rows: u16,
    },
    ListTabs,
    CloseTab {
        tab: String,
    },
    CloseAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerResponse {
    Ok,
    SnapshotText {
        tab: String,
        cols: u16,
        rows: u16,
        text: String,
        color: Option<SnapshotColorMetadata>,
    },
    FetchText {
        tab: String,
        text: String,
    },
    TabList {
        tabs: Vec<TabSummary>,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSummary {
    pub name: String,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub created_at_ms: u128,
    pub last_activity_at_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySpec {
    pub key: KeyCodeSpec,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl KeySpec {
    pub fn to_bytes(self) -> Result<Vec<u8>> {
        encode_key(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCodeSpec {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    Esc,
    Char(char),
    F(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButtonSpec {
    Left,
    Right,
    Middle,
}

impl MouseButtonSpec {
    pub fn as_xterm_code(self) -> u16 {
        match self {
            MouseButtonSpec::Left => 0,
            MouseButtonSpec::Middle => 1,
            MouseButtonSpec::Right => 2,
        }
    }
}

impl From<MouseButton> for MouseButtonSpec {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => MouseButtonSpec::Left,
            MouseButton::Right => MouseButtonSpec::Right,
            MouseButton::Middle => MouseButtonSpec::Middle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseEventSpec {
    Down {
        x: u16,
        y: u16,
        button: MouseButtonSpec,
    },
    Up {
        x: u16,
        y: u16,
        button: MouseButtonSpec,
    },
    Move {
        x: u16,
        y: u16,
    },
    Drag {
        x: u16,
        y: u16,
        button: MouseButtonSpec,
    },
    Wheel {
        x: Option<u16>,
        y: Option<u16>,
        delta_y: i16,
    },
}

impl MouseEventSpec {
    pub fn to_escape(self) -> Vec<u8> {
        match self {
            MouseEventSpec::Down { x, y, button } => {
                encode_sgr_mouse(button.as_xterm_code(), x, y, true)
            }
            MouseEventSpec::Up { x, y, .. } => encode_sgr_mouse(3, x, y, false),
            MouseEventSpec::Move { x, y } => encode_sgr_mouse(35, x, y, true),
            MouseEventSpec::Drag { x, y, button } => {
                encode_sgr_mouse(button.as_xterm_code() + 32, x, y, true)
            }
            MouseEventSpec::Wheel { x, y, delta_y } => {
                let x = x.unwrap_or(0);
                let y = y.unwrap_or(0);
                let base = if delta_y > 0 { 64 } else { 65 };
                encode_sgr_mouse(base, x, y, true)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModifierFlags {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl ModifierFlags {
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }
}

pub fn parse_key_spec(input: &str, explicit: &ModifierFlags) -> Result<KeySpec> {
    let mut spec = KeySpec {
        key: KeyCodeSpec::Char(' '),
        ctrl: explicit.ctrl,
        alt: explicit.alt,
        shift: explicit.shift,
        meta: explicit.meta,
    };

    let parts: Vec<&str> = input.split('+').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        bail!("key cannot be empty");
    }

    let mut key_part = None;
    for part in parts {
        let normalized = part.trim();
        if normalized.eq_ignore_ascii_case("ctrl") || normalized.eq_ignore_ascii_case("control") {
            spec.ctrl = true;
        } else if normalized.eq_ignore_ascii_case("alt") {
            spec.alt = true;
        } else if normalized.eq_ignore_ascii_case("shift") {
            spec.shift = true;
        } else if normalized.eq_ignore_ascii_case("meta")
            || normalized.eq_ignore_ascii_case("cmd")
            || normalized.eq_ignore_ascii_case("super")
        {
            spec.meta = true;
        } else {
            if key_part.is_some() {
                bail!("key specification `{input}` has multiple non-modifier segments");
            }
            key_part = Some(normalized.to_string());
        }
    }

    let key_raw =
        key_part.ok_or_else(|| anyhow!("key specification `{input}` is missing a key"))?;
    spec.key = parse_key_code(&key_raw)?;
    Ok(spec)
}

fn parse_key_code(input: &str) -> Result<KeyCodeSpec> {
    let lower = input.to_ascii_lowercase();
    let key = match lower.as_str() {
        "space" => KeyCodeSpec::Char(' '),
        "enter" => KeyCodeSpec::Enter,
        "tab" => KeyCodeSpec::Tab,
        "backtab" => KeyCodeSpec::BackTab,
        "backspace" => KeyCodeSpec::Backspace,
        "esc" | "escape" => KeyCodeSpec::Esc,
        "left" => KeyCodeSpec::Left,
        "right" => KeyCodeSpec::Right,
        "up" => KeyCodeSpec::Up,
        "down" => KeyCodeSpec::Down,
        "home" => KeyCodeSpec::Home,
        "end" => KeyCodeSpec::End,
        "pageup" => KeyCodeSpec::PageUp,
        "pagedown" => KeyCodeSpec::PageDown,
        "delete" | "del" => KeyCodeSpec::Delete,
        "insert" | "ins" => KeyCodeSpec::Insert,
        value if value.starts_with('f') => {
            let index = value[1..].parse::<u8>()?;
            KeyCodeSpec::F(index)
        }
        _ => {
            let mut chars = input.chars();
            let character = chars
                .next()
                .ok_or_else(|| anyhow!("key specification cannot be empty"))?;
            if chars.next().is_some() {
                bail!("unsupported key code `{input}`");
            }
            KeyCodeSpec::Char(character)
        }
    };
    Ok(key)
}

fn encode_key(spec: KeySpec) -> Result<Vec<u8>> {
    if spec.meta {
        bail!("meta key encoding is not implemented for v0");
    }

    let mut output = Vec::new();
    if spec.alt {
        output.push(0x1b);
    }

    match spec.key {
        KeyCodeSpec::Char(ch) => {
            if spec.ctrl {
                let lower = ch.to_ascii_lowercase();
                if !lower.is_ascii_alphabetic() {
                    bail!("Ctrl chord currently only supports alphabetic keys");
                }
                let byte = (lower as u8) - b'a' + 1;
                output.push(byte);
            } else {
                let rendered = if spec.shift {
                    ch.to_ascii_uppercase()
                } else {
                    ch
                };
                let mut buf = [0u8; 4];
                let encoded = rendered.encode_utf8(&mut buf);
                output.extend_from_slice(encoded.as_bytes());
            }
        }
        KeyCodeSpec::Enter => output.push(b'\r'),
        KeyCodeSpec::Tab => output.push(b'\t'),
        KeyCodeSpec::BackTab => output.extend_from_slice(b"\x1b[Z"),
        KeyCodeSpec::Backspace => output.push(0x08),
        KeyCodeSpec::Esc => output.push(0x1b),
        KeyCodeSpec::Left => output.extend_from_slice(b"\x1b[D"),
        KeyCodeSpec::Right => output.extend_from_slice(b"\x1b[C"),
        KeyCodeSpec::Up => output.extend_from_slice(b"\x1b[A"),
        KeyCodeSpec::Down => output.extend_from_slice(b"\x1b[B"),
        KeyCodeSpec::Home => output.extend_from_slice(b"\x1b[H"),
        KeyCodeSpec::End => output.extend_from_slice(b"\x1b[F"),
        KeyCodeSpec::PageUp => output.extend_from_slice(b"\x1b[5~"),
        KeyCodeSpec::PageDown => output.extend_from_slice(b"\x1b[6~"),
        KeyCodeSpec::Delete => output.extend_from_slice(b"\x1b[3~"),
        KeyCodeSpec::Insert => output.extend_from_slice(b"\x1b[2~"),
        KeyCodeSpec::F(index) => {
            let code = match index {
                1 => b"\x1bOP".as_slice(),
                2 => b"\x1bOQ".as_slice(),
                3 => b"\x1bOR".as_slice(),
                4 => b"\x1bOS".as_slice(),
                5 => b"\x1b[15~".as_slice(),
                6 => b"\x1b[17~".as_slice(),
                7 => b"\x1b[18~".as_slice(),
                8 => b"\x1b[19~".as_slice(),
                9 => b"\x1b[20~".as_slice(),
                10 => b"\x1b[21~".as_slice(),
                11 => b"\x1b[23~".as_slice(),
                12 => b"\x1b[24~".as_slice(),
                _ => bail!("unsupported function key F{index}"),
            };
            output.extend_from_slice(code);
        }
    }

    Ok(output)
}

fn encode_sgr_mouse(code: u16, x: u16, y: u16, down: bool) -> Vec<u8> {
    let suffix = if down { 'M' } else { 'm' };
    format!("\x1b[<{};{};{}{}", code, x + 1, y + 1, suffix).into_bytes()
}

fn normalize_theme_name(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

impl fmt::Display for KeyCodeSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCodeSpec::Backspace => write!(f, "Backspace"),
            KeyCodeSpec::Enter => write!(f, "Enter"),
            KeyCodeSpec::Left => write!(f, "Left"),
            KeyCodeSpec::Right => write!(f, "Right"),
            KeyCodeSpec::Up => write!(f, "Up"),
            KeyCodeSpec::Down => write!(f, "Down"),
            KeyCodeSpec::Home => write!(f, "Home"),
            KeyCodeSpec::End => write!(f, "End"),
            KeyCodeSpec::PageUp => write!(f, "PageUp"),
            KeyCodeSpec::PageDown => write!(f, "PageDown"),
            KeyCodeSpec::Tab => write!(f, "Tab"),
            KeyCodeSpec::BackTab => write!(f, "BackTab"),
            KeyCodeSpec::Delete => write!(f, "Delete"),
            KeyCodeSpec::Insert => write!(f, "Insert"),
            KeyCodeSpec::Esc => write!(f, "Esc"),
            KeyCodeSpec::Char(ch) => write!(f, "{ch}"),
            KeyCodeSpec::F(index) => write!(f, "F{index}"),
        }
    }
}

impl From<KeyCode> for KeyCodeSpec {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Backspace => KeyCodeSpec::Backspace,
            KeyCode::Enter => KeyCodeSpec::Enter,
            KeyCode::Left => KeyCodeSpec::Left,
            KeyCode::Right => KeyCodeSpec::Right,
            KeyCode::Up => KeyCodeSpec::Up,
            KeyCode::Down => KeyCodeSpec::Down,
            KeyCode::Home => KeyCodeSpec::Home,
            KeyCode::End => KeyCodeSpec::End,
            KeyCode::PageUp => KeyCodeSpec::PageUp,
            KeyCode::PageDown => KeyCodeSpec::PageDown,
            KeyCode::Tab => KeyCodeSpec::Tab,
            KeyCode::BackTab => KeyCodeSpec::BackTab,
            KeyCode::Delete => KeyCodeSpec::Delete,
            KeyCode::Insert => KeyCodeSpec::Insert,
            KeyCode::Esc => KeyCodeSpec::Esc,
            KeyCode::F(value) => KeyCodeSpec::F(value),
            KeyCode::Char(ch) => KeyCodeSpec::Char(ch),
            _ => KeyCodeSpec::Esc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ctrl_a_chord_and_flags_to_same_spec() {
        let chord = parse_key_spec("Ctrl+A", &ModifierFlags::empty()).unwrap();
        let flags = parse_key_spec(
            "A",
            &ModifierFlags {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
        )
        .unwrap();
        assert_eq!(chord, flags);
        assert_eq!(chord.key, KeyCodeSpec::Char('A'));
        assert!(chord.ctrl);
    }

    #[test]
    fn click_and_drag_expand_to_mouse_escape_sequences() {
        let down = MouseEventSpec::Down {
            x: 12,
            y: 4,
            button: MouseButtonSpec::Left,
        }
        .to_escape();
        let up = MouseEventSpec::Up {
            x: 12,
            y: 4,
            button: MouseButtonSpec::Left,
        }
        .to_escape();
        let drag = MouseEventSpec::Drag {
            x: 12,
            y: 4,
            button: MouseButtonSpec::Left,
        }
        .to_escape();
        assert_eq!(String::from_utf8(down).unwrap(), "\u{1b}[<0;13;5M");
        assert_eq!(String::from_utf8(up).unwrap(), "\u{1b}[<3;13;5m");
        assert_eq!(String::from_utf8(drag).unwrap(), "\u{1b}[<32;13;5M");
    }

    #[test]
    fn parse_space_key_alias() {
        let spec = parse_key_spec("Space", &ModifierFlags::empty()).unwrap();
        assert_eq!(spec.key, KeyCodeSpec::Char(' '));
    }

    #[test]
    fn snapshot_request_defaults_to_plain_text_render_mode_when_omitted() {
        let request: ClientRequest =
            serde_json::from_str(r#"{"Snapshot":{"tab":"demo","wait_stable_ms":1,"color":null}}"#)
                .unwrap();

        let ClientRequest::Snapshot { render, .. } = request else {
            panic!("expected snapshot request");
        };
        assert_eq!(render, SnapshotRenderMode::PlainText);
    }
}
