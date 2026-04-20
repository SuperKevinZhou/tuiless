# tuiless

`tuiless` is an experimental CLI for driving stateful terminal sessions from stateless commands.

The core idea is:

- every CLI invocation is short-lived;
- a per-workspace background runtime owns the actual shell tabs;
- commands such as `exec`, `press`, `type`, `resize`, `snapshot`, and `fetch` talk to that runtime over local IPC;
- `snapshot` returns the current visible terminal viewport as plain text.

This is intentionally similar in spirit to browser automation CLIs, but the target is a terminal tab rather than a DOM page. In v0, the only supported resource is a named shell tab.

## Status

This project is in an early v0 prototype state.

Working core path:

- auto-start a background runtime for the current working directory;
- create or reuse a named shell tab;
- execute shell input with `exec`;
- capture the current viewport with `snapshot`;
- fetch the full terminal text history with `fetch`;
- resize a tab;
- list open tabs.

Known rough edges:

- Windows is the only currently implemented IPC target.
- `attach` exists as a minimal polling-based interactive view and still needs broader real-TUI testing.
- mouse commands inject terminal mouse escape sequences, but target applications must enable mouse reporting.
- integration tests are not yet complete.

## Install / Build

```powershell
cargo build
```

The development binary is:

```powershell
.\target\debug\tuiless.exe
```

## Quick Start

Open a named tab with an explicit simulated terminal size:

```powershell
.\target\debug\tuiless.exe open tab_1 --cols 100 --rows 30
```

Run a shell command inside that tab:

```powershell
.\target\debug\tuiless.exe exec tab_1 "echo hello"
```

Snapshot the current visible viewport:

```powershell
.\target\debug\tuiless.exe snapshot tab_1
```

Fetch the full text content accumulated in the tab:

```powershell
.\target\debug\tuiless.exe fetch tab_1
```

List tabs in the current workspace runtime:

```powershell
.\target\debug\tuiless.exe list
```

Resize the simulated terminal:

```powershell
.\target\debug\tuiless.exe resize tab_1 --cols 120 --rows 40
```

Clean up the runtime:

```powershell
.\target\debug\tuiless.exe close --all
```

## Runtime Model

`tuiless` computes a session key from the canonical current working directory. The first CLI command for that workspace starts a background runtime. Later CLI commands connect to the same runtime and operate on the same tab state.

Runtime state is intentionally workspace-local:

- registry files live under `.tuiless/`;
- tab names are scoped to the current workspace;
- `tab_1` in one workspace is independent from `tab_1` in another workspace.

## Command Reference

### `open`

```powershell
tuiless open <tab> [--cols <n>] [--rows <n>]
```

Ensures a tab exists. If the tab is new, it is created with the requested terminal size, or the default size if no size is provided.

### `snapshot`

```powershell
tuiless snapshot <tab> [--wait-stable <ms>]
```

Returns the current visible viewport as plain text. This is not a full scrollback dump.

The default stable wait is currently `150ms`.

### `fetch`

```powershell
tuiless fetch <tab> [--wait-stable <ms>]
```

Returns the full plain-text contents currently retained for the tab, including scrollback history rather than only the visible viewport.

This command does not apply any truncation of its own.

The default stable wait is currently `150ms`.

### `exec`

```powershell
tuiless exec <tab> <line>
```

Types `<line>` into the tab and presses Enter.

This is a shell-friendly convenience command. It does not mean "spawn a managed subprocess and wait for completion".

### `type`

```powershell
tuiless type <tab> <text>
```

Injects printable text into the tab without pressing Enter.

### `press`

```powershell
tuiless press <tab> <key> [--ctrl] [--alt] [--shift] [--meta]
```

Sends a key event. Both chord syntax and explicit modifier flags are supported.

Examples:

```powershell
tuiless press tab_1 Enter
tuiless press tab_1 Ctrl+A
tuiless press tab_1 A --ctrl
tuiless press tab_1 Esc
tuiless press tab_1 Up
```

### Mouse Commands

Mouse commands use terminal cell coordinates. The origin is the top-left cell, with zero-based `x` and `y`.

```powershell
tuiless click <tab> --x <col> --y <row> [--button left]
tuiless drag <tab> --from-x <c1> --from-y <r1> --to-x <c2> --to-y <r2> [--button left]
tuiless wheel <tab> --delta-y <n> [--x <col> --y <row>]
tuiless mouse-down <tab> --x <col> --y <row> [--button left]
tuiless mouse-up <tab> --x <col> --y <row> [--button left]
tuiless mouse-move <tab> --x <col> --y <row>
```

These commands guarantee event injection only. Whether the target application reacts depends on terminal mouse reporting support.
For negative wheel values, both `--delta-y -3` and `--delta-y=-3` are accepted.

### `resize`

```powershell
tuiless resize <tab> --cols <n> --rows <n>
```

Updates the tab's simulated terminal size.

### `attach`

```powershell
tuiless attach <tab>
```

Starts a minimal interactive terminal view for a tab. Detach with `Ctrl+]`.

On start, `attach` syncs the tab to the current terminal size, enters an alternate-screen raw terminal view, enables mouse capture, and forwards keyboard, mouse, and resize events to the tab.

This is currently a polling implementation and still needs broader validation against full-screen TUIs.

### `list`

```powershell
tuiless list
```

Lists known tabs for the current workspace runtime.

### `close`

```powershell
tuiless close <tab>
tuiless close --all
```

Closes one tab or shuts down the current workspace runtime.

## Design Notes

The public model is deliberately simple:

- a tab is a shell-backed PTY session;
- terminal size is part of tab state;
- snapshots are viewport-oriented;
- fetch returns the retained full text history for a tab;
- raw input events and convenience actions both exist;
- higher-level locator APIs such as `click text=Submit` are intentionally out of scope for v0.

Internally, the current implementation uses:

- `clap` for CLI parsing;
- `tokio` named pipes for Windows IPC;
- `portable-pty` for PTY management;
- `vt100` for terminal screen parsing;
- `crossterm` for the minimal attach surface.

## Development Notes

When validating runtime behavior, run dependent commands serially. Do not run `open`, `exec`, `snapshot`, and process cleanup in parallel, because they target the same background runtime and can create misleading races.

Useful smoke sequence:

```powershell
Get-Process tuiless -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Remove-Item .tuiless -Recurse -Force -ErrorAction SilentlyContinue
cargo build
.\target\debug\tuiless.exe open smoke --cols 100 --rows 30
.\target\debug\tuiless.exe exec smoke "echo smoke-ok"
Start-Sleep -Milliseconds 800
.\target\debug\tuiless.exe snapshot smoke
```

Run tests:

```powershell
cargo test
```
