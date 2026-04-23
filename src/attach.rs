use std::io::{IsTerminal, stdin};
use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
    },
    execute, terminal,
};

use crate::app::RuntimeClient;
use crate::protocol::{KeyCodeSpec, MouseButtonSpec, MouseEventSpec};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(8);
const RENDER_INTERVAL: Duration = Duration::from_millis(16);
const LIVE_SNAPSHOT_WAIT_MS: u64 = 1;

pub fn ensure_interactive_terminal() -> Result<()> {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        bail!("attach requires an interactive terminal (TTY stdin/stdout)");
    }
    Ok(())
}

pub async fn attach(client: &RuntimeClient, tab: &str, wait_stable_ms: u64) -> Result<()> {
    ensure_interactive_terminal()?;

    let (cols, rows) = terminal::size()?;
    client.resize(tab, cols, rows).await?;
    let mut last_frame = client
        .snapshot_ansi_text(tab, wait_stable_ms.max(LIVE_SNAPSHOT_WAIT_MS))
        .await?;

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;

    if !last_frame.is_empty() {
        stdout.write_all(last_frame.as_bytes())?;
        stdout.flush()?;
    }

    let result = attach_loop(client, tab, &mut last_frame).await;

    let _ = execute!(
        stdout,
        cursor::Show,
        DisableMouseCapture,
        terminal::LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
    result
}

async fn attach_loop(client: &RuntimeClient, tab: &str, last_frame: &mut String) -> Result<()> {
    let mut next_render_at = Instant::now() + RENDER_INTERVAL;

    loop {
        let now = Instant::now();
        let timeout = if now >= next_render_at {
            Duration::ZERO
        } else {
            (next_render_at - now).min(EVENT_POLL_INTERVAL)
        };

        if event::poll(timeout)? {
            if handle_event(client, tab, event::read()?).await? {
                break;
            }
            while event::poll(Duration::ZERO)? {
                if handle_event(client, tab, event::read()?).await? {
                    return Ok(());
                }
            }
        }

        if Instant::now() >= next_render_at {
            render_snapshot(client, tab, LIVE_SNAPSHOT_WAIT_MS, last_frame).await?;
            next_render_at = Instant::now() + RENDER_INTERVAL;
        }
    }

    Ok(())
}

async fn handle_event(client: &RuntimeClient, tab: &str, event: Event) -> Result<bool> {
    match event {
        Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char(']'))
            {
                return Ok(true);
            }

            let spec = crate::protocol::KeySpec {
                key: KeyCodeSpec::from(key.code),
                ctrl: key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL),
                alt: key.modifiers.contains(crossterm::event::KeyModifiers::ALT),
                shift: key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT),
                meta: key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SUPER),
            };
            client.press(tab, spec).await?;
        }
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(button) => {
                client
                    .mouse_event(
                        tab,
                        MouseEventSpec::Down {
                            x: mouse.column,
                            y: mouse.row,
                            button: MouseButtonSpec::from(button),
                        },
                    )
                    .await?;
            }
            MouseEventKind::Up(button) => {
                client
                    .mouse_event(
                        tab,
                        MouseEventSpec::Up {
                            x: mouse.column,
                            y: mouse.row,
                            button: MouseButtonSpec::from(button),
                        },
                    )
                    .await?;
            }
            MouseEventKind::Drag(button) => {
                client
                    .mouse_event(
                        tab,
                        MouseEventSpec::Drag {
                            x: mouse.column,
                            y: mouse.row,
                            button: MouseButtonSpec::from(button),
                        },
                    )
                    .await?;
            }
            MouseEventKind::Moved => {
                client
                    .mouse_event(
                        tab,
                        MouseEventSpec::Move {
                            x: mouse.column,
                            y: mouse.row,
                        },
                    )
                    .await?;
            }
            MouseEventKind::ScrollUp => {
                client
                    .mouse_event(
                        tab,
                        MouseEventSpec::Wheel {
                            x: Some(mouse.column),
                            y: Some(mouse.row),
                            delta_y: 1,
                        },
                    )
                    .await?;
            }
            MouseEventKind::ScrollDown => {
                client
                    .mouse_event(
                        tab,
                        MouseEventSpec::Wheel {
                            x: Some(mouse.column),
                            y: Some(mouse.row),
                            delta_y: -1,
                        },
                    )
                    .await?;
            }
            MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {}
        },
        Event::Resize(cols, rows) => {
            client.resize(tab, cols, rows).await?;
        }
        _ => {}
    }
    Ok(false)
}

async fn render_snapshot(
    client: &RuntimeClient,
    tab: &str,
    wait_stable_ms: u64,
    last_frame: &mut String,
) -> Result<()> {
    let frame = client.snapshot_ansi_text(tab, wait_stable_ms).await?;
    if frame == *last_frame {
        return Ok(());
    }
    let mut stdout = stdout();
    stdout.write_all(frame.as_bytes())?;
    stdout.flush()?;
    *last_frame = frame;
    Ok(())
}
