# Detailed Documentation

## Table of Contents

1. Runtime model
2. Command families
3. Observation semantics
4. Troubleshooting
5. Validation and CI

## 1. Runtime Model

`tuiless` uses short-lived CLI invocations over a long-lived per-workspace background runtime.

- Compute a session key from canonical CWD.
- Auto-start runtime when needed.
- Route later commands from the same workspace to that runtime.
- Scope tabs by workspace session key.

Registry defaults:

- Windows default registry root: `%LOCALAPPDATA%\tuiless\registry`
- Optional override: `TUILESS_REGISTRY_DIR`

## 2. Command Families

Lifecycle and structure:

- `open <tab> [--cols <n>] [--rows <n>]`
- `list`
- `close <tab>` or `close --all`
- `resize <tab> --cols <n> --rows <n>`

Shell interaction:

- `exec <tab> <line>`: type line and press Enter.
- `type <tab> <text>`: type only.
- `press <tab> <key> [--ctrl] [--alt] [--shift] [--meta]`

Mouse interaction:

- `click`, `drag`, `wheel`, `mouse-down`, `mouse-up`, `mouse-move`
- Use cell coordinates with top-left origin `(0, 0)`.

Observation:

- `snapshot <tab> [--wait-stable <ms>]`
- `fetch <tab> [--wait-stable <ms>]`

Interactive terminal handoff:

- `attach <tab> [--wait-stable <ms>]`
- Require interactive TTY stdin and stdout.

## 3. Observation Semantics

Use `snapshot` when validating current viewport content.

- Return plain text viewport.
- Useful for deterministic point-in-time checks.

Use `fetch` when validating accumulated terminal history.

- Return retained full text history for the tab.
- Keep bounded history frames for fullscreen/alternate-screen TUIs.
- Prefer this command to inspect earlier content after heavy TUI activity.

## 4. Troubleshooting

Binary locked during build on Windows:

- Symptom: access denied for `tuiless.exe`.
- Action: stop running `tuiless` processes, then rebuild.

No output or unstable interactive evidence:

- Symptom: single long script misses transitions.
- Action: run commands in small serialized steps and capture evidence after each step.

`attach` fails in scripted environment:

- Symptom: immediate failure when no TTY.
- Action: use `snapshot`/`fetch` for automation; use `attach` only in real interactive terminal.

Wheel parsing issues:

- Prefer typed integer values, including negative deltas such as `--delta-y -3`.

## 5. Validation and CI

Run local checks before pushing:

```powershell
cargo fmt
cargo check --all-targets
cargo test
```

Recommended CI watch loop:

1. Push branch or `master`.
2. Inspect latest runs with `gh run list`.
3. Watch specific run with `gh run watch <run-id> --exit-status`.
4. Fix and repush until green.
