use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "tuiless-integration-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn run(exe: &Path, cwd: &Path, registry: &Path, args: &[&str]) -> String {
    let status = Command::new(exe)
        .current_dir(cwd)
        .env("TUILESS_REGISTRY_DIR", registry)
        .args(args)
        .status()
        .expect("tuiless command should start");
    assert!(
        status.success(),
        "command {:?} failed with status {status}",
        args
    );
    String::new()
}

fn capture(exe: &Path, cwd: &Path, registry: &Path, args: &[&str]) -> String {
    let output = Command::new(exe)
        .current_dir(cwd)
        .env("TUILESS_REGISTRY_DIR", registry)
        .args(args)
        .output()
        .expect("tuiless command should start");
    assert!(
        output.status.success(),
        "command {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn cleanup(exe: &Path, cwd: &Path, registry: &Path) {
    let _ = Command::new(exe)
        .current_dir(cwd)
        .env("TUILESS_REGISTRY_DIR", registry)
        .args(["close", "--all"])
        .status();
    let _ = fs::remove_dir_all(cwd);
    let _ = fs::remove_dir_all(registry);
}

#[test]
fn runtime_commands_cover_stateful_smoke_flow() {
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_tuiless"));
    let cwd = unique_dir("flow");
    let registry = unique_dir("registry");
    fs::create_dir_all(&cwd).unwrap();
    fs::create_dir_all(&registry).unwrap();

    run(
        &exe,
        &cwd,
        &registry,
        &["open", "smoke", "--cols", "100", "--rows", "30"],
    );
    run(&exe, &cwd, &registry, &["exec", "smoke", "echo smoke-ok"]);
    let snapshot = capture(
        &exe,
        &cwd,
        &registry,
        &["snapshot", "smoke", "--wait-stable", "30"],
    );
    assert!(snapshot.contains("smoke-ok"), "snapshot was: {snapshot:?}");

    run(&exe, &cwd, &registry, &["type", "smoke", "echo typed-ok"]);
    run(&exe, &cwd, &registry, &["press", "smoke", "Enter"]);
    let fetched = capture(
        &exe,
        &cwd,
        &registry,
        &["fetch", "smoke", "--wait-stable", "30"],
    );
    assert!(fetched.contains("typed-ok"), "fetch was: {fetched:?}");

    run(
        &exe,
        &cwd,
        &registry,
        &["resize", "smoke", "--cols", "120", "--rows", "40"],
    );
    let tabs = capture(&exe, &cwd, &registry, &["list"]);
    assert!(tabs.contains("smoke\t"), "list was: {tabs:?}");
    assert!(tabs.contains("120x40"), "list was: {tabs:?}");
    cleanup(&exe, &cwd, &registry);
}

#[test]
fn runtimes_are_isolated_by_workspace() {
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_tuiless"));
    let first = unique_dir("first");
    let second = unique_dir("second");
    let registry = unique_dir("isolation-registry");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::create_dir_all(&registry).unwrap();

    run(
        &exe,
        &first,
        &registry,
        &["exec", "same", "echo first-only"],
    );
    run(
        &exe,
        &second,
        &registry,
        &["exec", "same", "echo second-only"],
    );
    let first_text = capture(&exe, &first, &registry, &["fetch", "same"]);
    let second_text = capture(&exe, &second, &registry, &["fetch", "same"]);
    assert!(first_text.contains("first-only"));
    assert!(!first_text.contains("second-only"));
    assert!(second_text.contains("second-only"));
    assert!(!second_text.contains("first-only"));

    let _ = Command::new(&exe)
        .current_dir(&first)
        .env("TUILESS_REGISTRY_DIR", &registry)
        .args(["close", "--all"])
        .status();
    let _ = Command::new(&exe)
        .current_dir(&second)
        .env("TUILESS_REGISTRY_DIR", &registry)
        .args(["close", "--all"])
        .status();
    let _ = fs::remove_dir_all(&second);
    let _ = fs::remove_dir_all(&registry);
}
