# Repository Guidelines

## Project Structure & Module Organization
`tuiless` is a single-crate Rust CLI. Core code lives in `src/`, with `main.rs` wiring commands and modules such as `cli.rs`, `runtime.rs`, `session.rs`, `screen.rs`, and `protocol.rs` handling command parsing, background runtime state, PTY sessions, snapshots, and IPC. Project notes live in `README.md` and `TODO.md`. Build artifacts go to `target/`. Local runtime state is intentionally workspace-scoped and written to `.tuiless/` plus `.tuiless-runtime.log`; both are ignored and should not be committed.

## Build, Test, and Development Commands
- `cargo build`: compile the debug binary.
- `cargo run -- <command>`: run the CLI without manually locating the binary.
- `cargo test`: run the current test suite.
- `cargo fmt`: format Rust sources before committing.
- `cargo clippy --all-targets --all-features`: catch common Rust issues before opening a PR.

Useful smoke flow:

```powershell
cargo build
.\target\debug\tuiless.exe open smoke --cols 100 --rows 30
.\target\debug\tuiless.exe exec smoke "echo smoke-ok"
.\target\debug\tuiless.exe snapshot smoke
```

Run stateful commands serially, not in parallel, because they share one workspace runtime.

## Coding Style & Naming Conventions
Follow standard Rust style: 4-space indentation, `snake_case` for functions/modules/files, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep modules focused on one responsibility and prefer small helper functions over large command handlers. Use `cargo fmt` as the formatting source of truth.

## Testing Guidelines
This repository currently relies on `cargo test` plus manual smoke checks from `README.md`. Add unit tests near the relevant module with `#[cfg(test)]` when behavior is self-contained. For runtime or IPC scenarios, prefer deterministic integration tests that clean up `.tuiless/` before and after execution.

Name tests after the behavior they prove, for example `snapshot_returns_visible_viewport`.

## Commit & Pull Request Guidelines
Recent history uses short, imperative English commit subjects such as `Capture shell output in snapshots` and `Clean up runtime dependencies and warnings`. Keep that pattern. Group related changes into one commit and explain user-visible behavior in the PR description.

PRs should include:
- a short summary of what changed and why;
- the commands you ran (`cargo test`, smoke steps, lint/format checks);
- screenshots or terminal transcripts when CLI behavior changed;
- linked issues or TODO items when applicable.

## Runtime & Configuration Notes
This project is currently Windows-first. Validate changes against the workspace-local runtime model: commands operate on tabs scoped to the current directory, and cleanup should leave no tracked `.tuiless/` artifacts behind.
