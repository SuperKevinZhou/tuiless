# TODO

This file tracks the remaining work to complete the v0 plan.

## v0 Completion Checklist

- [x] Add stateless CLI command surface.
- [x] Auto-start a per-workspace background runtime.
- [x] Maintain named shell tabs in the runtime.
- [x] Support explicit tab creation with `open --cols --rows`.
- [x] Support lazy tab creation for tab-targeting commands.
- [x] Implement `exec` as `type line` plus Enter.
- [x] Capture current viewport text with `snapshot`.
- [x] Add `fetch` to return full retained tab text including scrollback.
- [x] Store tab terminal size and report it in `list`.
- [x] Implement `resize` against the PTY and screen parser.
- [x] Add key parsing for chord syntax such as `Ctrl+A`.
- [x] Add raw and convenience mouse command surfaces.
- [x] Add unit tests for key parsing, mouse event encoding, session keys, and screen basics.
- [x] Make `close --all` a clean request/response shutdown instead of a direct process exit.
- [ ] Add integration tests for `open -> exec -> snapshot`.
- [ ] Add integration tests for `open -> exec -> fetch`.
- [ ] Add integration tests for `type -> press Enter -> snapshot`.
- [ ] Add integration tests for `resize -> list -> snapshot`.
- [ ] Add integration tests for workspace isolation.
- [ ] Validate `press` behavior for Enter, Esc, arrows, Ctrl chords, and Alt chords.
- [ ] Validate `type` separately from `exec`.
- [ ] Validate mouse event injection against a real mouse-reporting TUI.
- [ ] Validate `attach` interactively, including detach with `Ctrl+]`.
- [ ] Decide whether v0 is Windows-first or add Unix domain socket support before calling it complete.

## Runtime / IPC

- [x] Replace `close --all`'s direct `process::exit(0)` with an orderly shutdown path:
  - send `Ok`;
  - stop accepting new connections;
  - drop tabs and PTYs;
  - delete the registry file;
  - exit after response flush.
- [ ] Add stale registry recovery tests.
- [ ] Add same-session singleton tests so multiple runtimes cannot race and overwrite `.tuiless/<session>.json`.
- [ ] Make process health checking less Windows-command-dependent.
- [ ] Consider moving session registry out of the workspace once permissions and portability are better understood.

## PTY / Snapshot

- [ ] Confirm `snapshot` behavior for fullscreen/alternate-screen TUIs.
- [ ] Confirm viewport-only behavior under scrollback-heavy output.
- [ ] Confirm `fetch` behavior under alternate-screen/fullscreen TUIs.
- [ ] Add a structured debug mode for raw PTY bytes if future parser issues appear.
- [ ] Handle more terminal query responses beyond `ESC[6n` if real TUIs need them.
- [ ] Decide whether to expose `--json` snapshots in a later version.

## Input Events

- [ ] Add tests for `exec` expansion semantics.
- [ ] Add tests for `click` expansion into down/up events.
- [ ] Add tests for `drag` interpolation.
- [ ] Review mouse wheel `delta-y` direction against common terminal expectations.
- [ ] Add support for more key encodings if needed:
  - Shift+arrows;
  - Ctrl+arrows;
  - function keys beyond F12;
  - non-alphabetic Ctrl chords.

## Attach

- [x] Replace the current attach loop with an event-priority update model and ANSI frame rendering.
- [x] Forward keyboard input from raw mode.
- [x] Forward terminal resize events.
- [x] Forward mouse events in attach mode.
- [ ] Validate attach interactively against a real shell session.
- [ ] Add a visible status/help hint for detach.
- [ ] Decide whether attach should use alternate screen or preserve normal terminal scrollback.

## Cross-Platform

- [ ] Implement Unix domain socket IPC.
- [ ] Verify `portable-pty` behavior on Linux.
- [ ] Verify `portable-pty` behavior on macOS.
- [ ] Normalize shell default selection per platform:
  - Windows: `cmd.exe`, `pwsh.exe`, or `powershell.exe`;
  - Unix: `$SHELL` or `/bin/sh`.
- [ ] Add CI matrix once the project has GitHub Actions.

## Documentation

- [x] Add initial README.
- [x] Add TODO list.
- [ ] Add examples for automating a fullscreen TUI.
- [ ] Add troubleshooting notes for stale runtimes and locked debug binaries.
- [ ] Add architecture notes for CLI/runtime/IPC/PTTY boundaries.
