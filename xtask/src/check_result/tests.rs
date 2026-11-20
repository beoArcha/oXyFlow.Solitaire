//! Tests of the check result file: format, parsing, errors and the file round trip.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::{CheckResult, CheckResultError, StepLog, StepRecord, parse_step, path, read, save};

fn step(name: &str, passed: bool, duration_ms: u128) -> StepRecord {
    StepRecord {
        name: name.to_owned(),
        passed,
        duration_ms,
    }
}

fn passing() -> CheckResult {
    CheckResult {
        commit: "0123abc".to_owned(),
        mode: "quick".to_owned(),
        passed: true,
        steps: vec![step("fmt", true, 12), step("test", true, 3400)],
        output: Vec::new(),
    }
}

fn failing() -> CheckResult {
    CheckResult {
        commit: "0123abc-dirty".to_owned(),
        mode: "full".to_owned(),
        passed: false,
        steps: vec![step("fmt", true, 12), step("clippy", false, 9000)],
        output: vec![
            "error[E0308]: mismatched types".to_owned(),
            "  --> src/lib.rs:1".to_owned(),
        ],
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xtask-check-{name}-{}", std::process::id()));
    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn passing_result_renders_the_documented_layout() {
    let text = passing().render();
    assert_eq!(
        text,
        "commit: 0123abc\nmode: quick\nresult: pass\nsteps:\n  fmt: pass 12 ms\n  test: pass 3400 ms\noutput:\n"
    );
}

#[test]
fn failing_result_renders_the_tail_after_output() {
    let text = failing().render();
    assert!(text.contains("result: fail\n"));
    assert!(text.contains("  clippy: fail 9000 ms\n"));
    assert!(text.ends_with("output:\nerror[E0308]: mismatched types\n  --> src/lib.rs:1\n"));
}

#[test]
fn render_then_parse_round_trips() {
    for result in [passing(), failing()] {
        let parsed = CheckResult::parse(&result.render()).unwrap();
        assert_eq!(parsed, result);
    }
}

#[test]
fn output_lines_that_look_like_headers_are_kept_verbatim() {
    let mut result = failing();
    result.output = vec!["output:".to_owned(), "steps: nope".to_owned()];
    let parsed = CheckResult::parse(&result.render()).unwrap();
    assert_eq!(parsed.output, result.output);
}

#[test]
fn parse_rejects_a_missing_output_header() {
    let text = "commit: a\nmode: quick\nresult: pass\nsteps:\n  fmt: pass 1 ms\n";
    assert!(matches!(
        CheckResult::parse(text),
        Err(CheckResultError::Malformed { .. })
    ));
}

#[test]
fn parse_rejects_an_unknown_status() {
    let text = "commit: a\nmode: quick\nresult: maybe\nsteps:\noutput:\n";
    assert!(matches!(
        CheckResult::parse(text),
        Err(CheckResultError::Malformed { reason, .. }) if reason == "expected `pass` or `fail`"
    ));
}

#[test]
fn parse_rejects_a_step_without_a_duration() {
    assert!(parse_step("  fmt: pass").is_err());
    assert!(parse_step("  fmt: pass slow ms").is_err());
    assert!(parse_step("  fmt: pass 12 seconds").is_err());
    assert!(parse_step("fmt: pass 12 ms").is_err());
}

#[test]
fn parse_step_reads_name_status_and_duration() {
    assert_eq!(
        parse_step("  clippy: fail 9000 ms").unwrap(),
        step("clippy", false, 9000)
    );
}

#[test]
fn step_log_keeps_the_tail_of_the_failing_step_only() {
    let mut log = StepLog::default();
    log.record(
        "fmt",
        true,
        Duration::from_millis(5),
        vec!["ignored".to_owned()],
    );
    log.record(
        "clippy",
        false,
        Duration::from_millis(7),
        vec!["warning: x".to_owned()],
    );
    assert_eq!(log.steps.len(), 2);
    assert_eq!(log.output, vec!["warning: x".to_owned()]);
}

#[test]
fn step_log_uses_the_error_as_output_when_no_step_has_a_tail() {
    let mut log = StepLog::default();
    log.record("lint", false, Duration::from_millis(1), Vec::new());
    let failure = Some("lint found 2 error(s)".to_owned());
    let result = log.into_result("abc".to_owned(), "full", failure);
    assert!(!result.passed);
    assert_eq!(result.mode, "full");
    assert_eq!(result.output, vec!["lint found 2 error(s)".to_owned()]);
}

#[test]
fn step_log_of_a_passing_run_is_a_pass_without_output() {
    let mut log = StepLog::default();
    log.record("fmt", true, Duration::from_millis(5), Vec::new());
    let result = log.into_result("abc".to_owned(), "quick", None);
    assert!(result.passed);
    assert!(result.output.is_empty());
    assert_eq!(result.steps, vec![step("fmt", true, 5)]);
}

#[test]
fn save_and_read_use_target_check_result_txt() {
    let root = scratch("file");
    assert_eq!(path(&root), root.join("target").join("check-result.txt"));
    save(&root, &passing()).unwrap();
    assert_eq!(read(&root).unwrap(), passing());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn read_reports_a_missing_file() {
    let root = scratch("missing");
    assert!(matches!(read(&root), Err(CheckResultError::Missing { .. })));
    fs::remove_dir_all(&root).ok();
}
