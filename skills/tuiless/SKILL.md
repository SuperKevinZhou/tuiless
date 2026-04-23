---
name: tuiless
description: Operate, validate, and troubleshoot the tuiless Windows-first terminal automation CLI. Use when Codex needs to run or debug tab lifecycle commands (`open`, `exec`, `type`, `press`, `click`, `drag`, `wheel`, `snapshot`, `fetch`, `attach`, `close`), explain runtime/session behavior, produce usage docs, or guide reliable smoke/CI workflows for this repository.
---

# tuiless Skill

Use this skill to execute repository-grounded work on `tuiless` with consistent command semantics and validation discipline.

## Quick Start

1. Read [Basic Usage](references/basic-usage.md) to choose a runnable path quickly.
2. Use the command recipes from that file for immediate execution.
3. Read [Detailed Documentation](references/detailed-documentation.md) when command shape or behavior is ambiguous.
4. Apply [Best Practices](references/best-practices.md) before finalizing results or suggesting workflow changes.

## Workflow

1. Confirm execution context:
- Detect current working directory and ensure commands target the intended workspace.
- Assume runtime state is workspace-scoped and stateful.

2. Choose the task lane:
- Use command-operation lane for `open`/`exec`/`type`/`press`/mouse/`resize`/`attach`.
- Use observation lane for `snapshot` and `fetch`.
- Use maintenance lane for docs, tests, formatting, and CI release operations.

3. Execute serially for stateful operations:
- Run dependent runtime commands in sequence.
- Avoid parallel command orchestration for the same workspace runtime.

4. Validate behavior with the right artifact:
- Use `snapshot` for visible viewport checks.
- Use `fetch` for full retained text history checks.
- Prefer step-by-step evidence capture for interactive TUI verification.

5. Close with hygiene:
- Run format/test checks for code changes.
- Keep runtime artifacts out of tracked files.
- Report exact commands and user-visible outcomes.

## Reference Map

- [Basic Usage](references/basic-usage.md): fast start and common command examples.
- [Detailed Documentation](references/detailed-documentation.md): runtime model, command semantics, troubleshooting.
- [Best Practices](references/best-practices.md): reliability, testing discipline, and CI/release guardrails.

## Output Expectations

- Provide runnable commands, not abstract descriptions.
- Explain whether output reflects viewport (`snapshot`) or retained history (`fetch`).
- Highlight Windows-specific constraints when relevant.
- Include validation evidence (tests, smoke outputs, or CI status) for behavior-changing work.

## Boundary Rules

- Keep `serve` internal and avoid presenting it as a user-facing entrypoint.
- Preserve `fetch` semantics as user-visible scrollback/history, not raw PTY bytes.
- Avoid destructive cleanup outside the intended workspace.
- Keep command examples compatible with the repository's current command surface.

Use references directly instead of duplicating large command tables in this file.
