//! The result file of `cargo xtask check`: `target/check-result.txt`.
//!
//! The file is UTF-8 with LF line endings and has this shape:
//!
//! ```text
//! commit: <full commit SHA>[-dirty]
//! mode: full | quick | pr
//! result: pass | fail
//! steps:
//!   <name>: pass | fail <duration> ms
//! output:
//! <last lines of the failing step, verbatim; nothing when the check passed>
//! ```
//!
//! Steps appear in run order. A step that is never reached is not listed.
//! The `output:` line is the last header line; everything after it is output.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const OUTPUT_HEADER: &str = "output:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub commit: String,
    pub mode: String,
    pub passed: bool,
    pub steps: Vec<StepRecord>,
    pub output: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CheckResultError {
    #[error("`{path}` is missing; run `cargo xtask check` first")]
    Missing { path: PathBuf },
    #[error("cannot read or write `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot start `git`")]
    GitSpawn(#[source] io::Error),
    #[error("`git {command}` failed (exit code {code:?})")]
    GitFailed { command: String, code: Option<i32> },
    #[error("malformed check result: {reason} (line: `{line}`)")]
    Malformed { line: String, reason: &'static str },
}

impl CheckResult {
    pub fn render(&self) -> String {
        let mut lines = self.summary_lines();
        lines.push(OUTPUT_HEADER.to_owned());
        lines.extend(self.output.iter().cloned());
        lines.join("\n") + "\n"
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("commit: {}", self.commit),
            format!("mode: {}", self.mode),
            format!("result: {}", status_word(self.passed)),
            "steps:".to_owned(),
        ];
        lines.extend(self.steps.iter().map(|step| {
            format!(
                "  {}: {} {} ms",
                step.name,
                status_word(step.passed),
                step.duration_ms
            )
        }));
        lines
    }

    pub fn failed_step(&self) -> Option<&StepRecord> {
        self.steps.iter().find(|step| !step.passed)
    }

    pub fn parse(text: &str) -> Result<Self, CheckResultError> {
        let mut lines = text.lines();
        let commit = header(&mut lines, "commit")?;
        let mode = header(&mut lines, "mode")?;
        let passed = parse_status(&header(&mut lines, "result")?)?;
        expect_line(lines.next(), "steps:")?;
        let body: Vec<&str> = lines.collect();
        let split = body.iter().position(|line| *line == OUTPUT_HEADER).ok_or(
            CheckResultError::Malformed {
                line: String::new(),
                reason: "missing `output:` line",
            },
        )?;
        let steps = body[..split]
            .iter()
            .map(|line| parse_step(line))
            .collect::<Result<Vec<_>, _>>()?;
        let output = body[split + 1..]
            .iter()
            .map(|line| (*line).to_owned())
            .collect();
        Ok(Self {
            commit,
            mode,
            passed,
            steps,
            output,
        })
    }
}

fn status_word(passed: bool) -> &'static str {
    if passed { "pass" } else { "fail" }
}

fn parse_status(word: &str) -> Result<bool, CheckResultError> {
    match word {
        "pass" => Ok(true),
        "fail" => Ok(false),
        _ => Err(CheckResultError::Malformed {
            line: word.to_owned(),
            reason: "expected `pass` or `fail`",
        }),
    }
}

fn header<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
    key: &'static str,
) -> Result<String, CheckResultError> {
    let line = lines.next().unwrap_or_default();
    line.strip_prefix(key)
        .and_then(|rest| rest.strip_prefix(": "))
        .map(str::to_owned)
        .ok_or(CheckResultError::Malformed {
            line: line.to_owned(),
            reason: "expected a `key: value` header line",
        })
}

fn expect_line(line: Option<&str>, expected: &'static str) -> Result<(), CheckResultError> {
    match line {
        Some(found) if found == expected => Ok(()),
        other => Err(CheckResultError::Malformed {
            line: other.unwrap_or_default().to_owned(),
            reason: "expected the `steps:` line",
        }),
    }
}

fn parse_step(line: &str) -> Result<StepRecord, CheckResultError> {
    let malformed = |reason| CheckResultError::Malformed {
        line: line.to_owned(),
        reason,
    };
    let (name, value) = line
        .strip_prefix("  ")
        .and_then(|rest| rest.split_once(": "))
        .ok_or_else(|| malformed("expected `  <name>: <status> <ms> ms`"))?;
    let fields: Vec<&str> = value.split_whitespace().collect();
    match fields.as_slice() {
        [status, millis, "ms"] => Ok(StepRecord {
            name: name.to_owned(),
            passed: parse_status(status)?,
            duration_ms: millis
                .parse()
                .map_err(|_| malformed("duration is not a number"))?,
        }),
        _ => Err(malformed("expected `<status> <ms> ms` after the name")),
    }
}

pub fn path(root: &Path) -> PathBuf {
    root.join("target").join("check-result.txt")
}

pub fn save(root: &Path, result: &CheckResult) -> Result<(), CheckResultError> {
    let file = path(root);
    let directory = file.parent().map(Path::to_path_buf).unwrap_or_default();
    fs::create_dir_all(&directory).map_err(|source| CheckResultError::Io {
        path: directory,
        source,
    })?;
    fs::write(&file, result.render()).map_err(|source| CheckResultError::Io { path: file, source })
}

pub fn read(root: &Path) -> Result<CheckResult, CheckResultError> {
    let file = path(root);
    match fs::read_to_string(&file) {
        Ok(text) => CheckResult::parse(&text),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            Err(CheckResultError::Missing { path: file })
        }
        Err(source) => Err(CheckResultError::Io { path: file, source }),
    }
}

#[derive(Default)]
pub struct StepLog {
    steps: Vec<StepRecord>,
    output: Vec<String>,
}

impl StepLog {
    pub fn record(&mut self, name: &str, passed: bool, elapsed: Duration, tail: Vec<String>) {
        self.steps.push(StepRecord {
            name: name.to_owned(),
            passed,
            duration_ms: elapsed.as_millis(),
        });
        if !passed {
            self.output = tail;
        }
    }

    pub fn into_result(self, commit: String, mode: &str, failure: Option<String>) -> CheckResult {
        let passed = failure.is_none();
        let output = if self.output.is_empty() {
            failure
                .map(|message| message.lines().map(str::to_owned).collect())
                .unwrap_or_default()
        } else {
            self.output
        };
        CheckResult {
            commit,
            mode: mode.to_owned(),
            passed,
            steps: self.steps,
            output,
        }
    }
}

pub fn commit_id(root: &Path) -> Result<String, CheckResultError> {
    let sha = git(root, &["rev-parse", "HEAD"])?;
    let dirty = if git(root, &["status", "--porcelain"])?.trim().is_empty() {
        ""
    } else {
        "-dirty"
    };
    Ok(format!("{}{dirty}", sha.trim()))
}

fn git(root: &Path, args: &[&str]) -> Result<String, CheckResultError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(CheckResultError::GitSpawn)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(CheckResultError::GitFailed {
            command: args.join(" "),
            code: output.status.code(),
        })
    }
}

#[cfg(test)]
mod tests;
