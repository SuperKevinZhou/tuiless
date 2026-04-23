use std::io::{IsTerminal, stdin};
use std::io::{Write, stdout};
use std::time::Duration;

use anyhow::{Result, bail};
use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
    },
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

use crate::protocol::{KeyCodeSpec, MouseButtonSpec, MouseEventSpec};
use crate::{app::RuntimeClient, protocol::ServerResponse};

pub fn ensure_interactive_terminal() -> Result<()> {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        bail!("attach requires an interactive terminal (TTY stdin/stdout)");
    }
    Ok(())
}

pub async fn attach(client: &RuntimeClient, tab: &str, wait_stable_ms: u64) -> Result<()> {
    ensure_interactive_terminal()?;

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;

    let result = async {
        let (cols, rows) = terminal::size()?;
        client.resize(tab, cols, rows).await?;
        attach_loop(client, tab, wait_stable_ms).await
    }
    .await;

    let _ = execute!(
        stdout,
        cursor::Show,
        DisableMouseCapture,
        terminal::LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
    result
}

async fn attach_loop(client: &RuntimeClient, tab: &str, wait_stable_ms: u64) -> Result<()> {
    loop {
        render_snapshot(client, tab, wait_stable_ms).await?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char(']'))
                    {
                        break;
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
        }
    }
    Ok(())
}

async fn render_snapshot(client: &RuntimeClient, tab: &str, wait_stable_ms: u64) -> Result<()> {
    let response = client.snapshot_raw(tab, wait_stable_ms, None).await?;
    let ServerResponse::SnapshotText { text, .. } = response else {
        bail!("unexpected runtime response while attaching");
    };
    let mut stdout = stdout();
    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        Clear(ClearType::All),
        Print(text)
    )?;
    stdout.flush()?;
    Ok(())
}
