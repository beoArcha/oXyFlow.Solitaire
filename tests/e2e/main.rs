//! End-to-end tests of the built binary. Each test runs `oxysol` as a child process with
//! `OXYSOL_HOME` in a temporary folder. None of them starts the terminal interface.

use std::path::Path;
use std::process::{Command, Output};

use oxysol::logging::LEVEL_VARIABLE;
use oxysol::panic_hook::TEST_PANIC_VARIABLE;
use oxysol::paths::{HOME_VARIABLE, LOG_FILE};

fn run_oxysol(args: &[&str], home: &Path, extra: &[(&str, &str)]) -> Option<Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_oxysol"));
    command
        .args(args)
        .env(HOME_VARIABLE, home)
        .env_remove(TEST_PANIC_VARIABLE);
    command.env_remove(LEVEL_VARIABLE);
    for (name, value) in extra {
        command.env(name, value);
    }
    command.output().ok()
}

#[test]
fn version_prints_the_version_and_exits_zero() {
    let home = tempfile::tempdir().ok();
    let Some(home) = home else {
        return;
    };
    let output = run_oxysol(&["--version"], home.path(), &[]);
    let Some(output) = output else {
        panic!("the binary did not start");
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output.status);
    assert!(
        stdout.contains(&format!("oXySol {}", env!("OXYSOL_VERSION"))),
        "stdout was {stdout:?}"
    );
    assert!(
        stdout.contains(&format!("oXySol {}.", major_minor(&metadata_version()))),
        "stdout was {stdout:?}"
    );
}

#[test]
fn version_prints_the_copyright_and_the_licence_on_a_second_line() {
    let home = tempfile::tempdir().ok();
    let Some(home) = home else {
        return;
    };
    let Some(output) = run_oxysol(&["--version"], home.path(), &[("LC_ALL", "en_US.UTF-8")]) else {
        panic!("the binary did not start");
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "stdout was {stdout:?}");
    assert!(lines[1].contains(env!("CARGO_PKG_AUTHORS")), "{stdout:?}");
    assert!(lines[1].contains(env!("CARGO_PKG_LICENSE")), "{stdout:?}");
}

fn major_minor(version: &str) -> String {
    version.splitn(3, '.').take(2).collect::<Vec<_>>().join(".")
}

fn metadata_version() -> String {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("metadata.toml");
    let text = std::fs::read_to_string(manifest).unwrap_or_default();
    let table = toml::from_str::<toml::Table>(&text).unwrap_or_default();
    table
        .get("project")
        .and_then(|project| project.get("version"))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn help_prints_the_options_and_exits_zero() {
    let home = tempfile::tempdir().ok();
    let Some(home) = home else {
        return;
    };
    let output = run_oxysol(&["--help"], home.path(), &[]);
    let Some(output) = output else {
        panic!("the binary did not start");
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output.status);
    assert!(stdout.contains("--seed"), "stdout was {stdout:?}");
}

#[test]
fn an_unknown_argument_exits_with_two() {
    let home = tempfile::tempdir().ok();
    let Some(home) = home else {
        return;
    };
    let output = run_oxysol(&["--no-such-flag"], home.path(), &[]);
    let Some(output) = output else {
        panic!("the binary did not start");
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("--help"), "stderr was {stderr:?}");
}

#[test]
fn a_panic_leaves_a_log_record_and_a_failing_exit_code() {
    let home = tempfile::tempdir().ok();
    let Some(home) = home else {
        return;
    };
    let output = run_oxysol(&[], home.path(), &[(TEST_PANIC_VARIABLE, "1")]);
    let Some(output) = output else {
        panic!("the binary did not start");
    };
    assert!(!output.status.success(), "{:?}", output.status);
    let log = std::fs::read_to_string(home.path().join(LOG_FILE)).unwrap_or_default();
    assert!(log.contains("oXySol starting"), "log was {log:?}");
    assert!(log.contains("oxysol::critical"), "log was {log:?}");
    assert!(
        log.contains("panic requested by OXYSOL_TEST_PANIC"),
        "log was {log:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(LOG_FILE), "stderr was {stderr:?}");
}
