# Best Practices

## 1. Keep Stateful Flows Serial

Run stateful runtime commands one-by-one in the same workspace.

- Avoid parallel `open`/`exec`/`snapshot`/`fetch` pipelines.
- Avoid racing cleanup and action commands against each other.

## 2. Pick the Right Observation API

Use `snapshot` and `fetch` intentionally:

- Use `snapshot` for viewport assertions.
- Use `fetch` for full retained terminal history.
- Avoid interpreting `snapshot` as a full scrollback dump.

## 3. Validate TUI Interactions Incrementally

For TUI apps (for example lazygit-like flows):

- Perform one input action at a time (`press`, `click`, `wheel`, `type`).
- Capture `snapshot` or `fetch` immediately after each action.
- Store evidence in text files when debugging regressions.

## 4. Preserve Runtime Hygiene

- Keep registry/runtime data out of tracked repository files.
- Use `close --all` at the end of smoke scripts.
- Kill stray runtime processes before rebuild if Windows file locks appear.

## 5. Treat `serve` as Internal

- Do not teach normal users to call `serve` directly.
- Keep user docs focused on public commands.

## 6. Favor Small, Verifiable Changes

When changing CLI behavior:

- Add or update focused tests with behavior-centric names.
- Run `cargo fmt`, `cargo check --all-targets`, and `cargo test`.
- Describe user-visible behavior in commit messages and PR notes.

## 7. Protect Cross-Workspace Isolation

- Remember tab names are workspace-scoped.
- Verify current CWD before assuming existing tabs should appear.
- Re-run `open` in the target workspace when switching projects.

## 8. Release with Observable Gates

- Ensure CI is green before final release confidence.
- Tag after stable code state and checks.
- Verify both `master` CI and tag-triggered release workflow outcomes.
