//! Pull request state for the merged check, read with the `gh` CLI. Without a working `gh` the check
//! gives no answer and a branch is judged by git alone.

use std::path::Path;
use std::process::Command;

pub trait PullRequests {
    /// Whether the pull requests with this head branch are finished (closed or merged) and none is open.
    fn finished(&self, root: &Path, branch: &str) -> bool;
}

pub struct GhCli;

impl PullRequests for GhCli {
    fn finished(&self, root: &Path, branch: &str) -> bool {
        let output = Command::new("gh")
            .args([
                "pr",
                "list",
                "--state",
                "all",
                "--head",
                branch,
                "--json",
                "state",
                "--jq",
                ".[].state",
            ])
            .current_dir(root)
            .output();
        match output {
            Ok(output) if output.status.success() => {
                finished_states(&String::from_utf8_lossy(&output.stdout))
            }
            _ => false,
        }
    }
}

/// `true` when no pull request is `OPEN` and at least one is `MERGED` or `CLOSED`.
pub fn finished_states(text: &str) -> bool {
    let states: Vec<&str> = text.lines().map(str::trim).collect();
    !states.contains(&"OPEN")
        && states
            .iter()
            .any(|state| matches!(*state, "MERGED" | "CLOSED"))
}
