//! Git queries of the preflight. Every call runs `git` in the given directory.

use std::path::Path;
use std::process::{Command, Output};

use super::PreflightError;

pub fn run(dir: &Path, args: &[&str]) -> Result<Output, PreflightError> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(PreflightError::Spawn)
}

fn stdout(dir: &Path, args: &[&str]) -> Result<String, PreflightError> {
    let output = run(dir, args)?;
    if !output.status.success() {
        return Err(PreflightError::Git {
            command: args.join(" "),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    String::from_utf8(output.stdout).map_err(|_| PreflightError::NotUtf8 {
        command: args.join(" "),
    })
}

/// Commits the branch is behind (first) and ahead (second) of `base`.
pub fn divergence(dir: &Path, base: &str) -> Result<(usize, usize), PreflightError> {
    let range = format!("{base}...HEAD");
    let args = ["rev-list", "--left-right", "--count", range.as_str()];
    let text = stdout(dir, &args)?;
    parse_counts(&text).ok_or_else(|| PreflightError::Counts {
        command: args.join(" "),
    })
}

pub fn parse_counts(text: &str) -> Option<(usize, usize)> {
    let mut counts = text.split_whitespace();
    let behind = counts.next()?.parse().ok()?;
    let ahead = counts.next()?.parse().ok()?;
    Some((behind, ahead))
}

/// Files changed since the merge base with `base`, committed or not, plus untracked files. Sorted.
pub fn changed_files(dir: &Path, base: &str) -> Result<Vec<String>, PreflightError> {
    let merge_base = stdout(dir, &["merge-base", base, "HEAD"])?;
    let mut files = lines(&stdout(dir, &["diff", "--name-only", merge_base.trim()])?);
    files.extend(lines(&stdout(
        dir,
        &["ls-files", "--others", "--exclude-standard"],
    )?));
    files.sort();
    files.dedup();
    Ok(files)
}

fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}
