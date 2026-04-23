# Basic Usage

## Purpose

Use this guide to run `tuiless` quickly and correctly in local Windows workflows.

## Build

```powershell
cargo build
```

Use the debug binary:

```powershell
.\target\debug\tuiless.exe
```

## Fast Smoke Flow

Run this sequence serially:

```powershell
Get-Process tuiless -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
cargo build
.\target\debug\tuiless.exe open smoke --cols 100 --rows 30
.\target\debug\tuiless.exe exec smoke "echo smoke-ok"
Start-Sleep -Milliseconds 800
.\target\debug\tuiless.exe snapshot smoke
.\target\debug\tuiless.exe fetch smoke
.\target\debug\tuiless.exe close --all
```

## Core Command Examples

Open or reuse a tab:

```powershell
.\target\debug\tuiless.exe open demo --cols 120 --rows 40
```

Execute a shell line:

```powershell
.\target\debug\tuiless.exe exec demo "dir"
```

Inject plain text:

```powershell
.\target\debug\tuiless.exe type demo "git status"
.\target\debug\tuiless.exe press demo Enter
```

Capture viewport text:

```powershell
.\target\debug\tuiless.exe snapshot demo --wait-stable 150
```

Capture retained full history text:

```powershell
.\target\debug\tuiless.exe fetch demo --wait-stable 150
```

Inject mouse and keyboard events:

```powershell
.\target\debug\tuiless.exe press demo Ctrl+A
.\target\debug\tuiless.exe click demo --x 10 --y 5 --button left
.\target\debug\tuiless.exe wheel demo --delta-y -3 --x 10 --y 5
```

Resize tab:

```powershell
.\target\debug\tuiless.exe resize demo --cols 140 --rows 45
```

Interactive attach (TTY only):

```powershell
.\target\debug\tuiless.exe attach demo
```

## Command Result Interpretation

- Treat `snapshot` as current visible viewport only.
- Treat `fetch` as retained full terminal text history for that tab.
- Treat failed `attach` in non-interactive pipelines as expected behavior.
