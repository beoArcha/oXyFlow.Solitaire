//! `cargo xtask report`: prints the last check result as Markdown, ready to paste into a pull request body.

use std::path::Path;

use crate::check_result::{self, CheckResult};
use crate::error::XtaskError;

pub fn run(root: &Path) -> Result<(), XtaskError> {
    let result = check_result::read(root)?;
    print!("{}", markdown(&result));
    Ok(())
}

pub fn markdown(result: &CheckResult) -> String {
    let mut text = "## Check result\n\n".to_owned();
    text.push_str(&fence(&result.summary_lines()));
    if !result.passed && !result.output.is_empty() {
        text.push('\n');
        text.push_str(&output_heading(result));
        text.push_str(&fence(&result.output));
    }
    text
}

fn output_heading(result: &CheckResult) -> String {
    match result.failed_step() {
        Some(step) => format!("Output of the failing step `{}`:\n\n", step.name),
        None => "Output:\n\n".to_owned(),
    }
}

fn fence(lines: &[String]) -> String {
    let longest = lines
        .iter()
        .map(|line| longest_backtick_run(line))
        .max()
        .unwrap_or(0);
    let marker = "`".repeat(longest.max(2) + 1);
    format!("{marker}text\n{}\n{marker}\n", lines.join("\n"))
}

fn longest_backtick_run(line: &str) -> usize {
    line.split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{fence, markdown};
    use crate::check_result::{CheckResult, StepRecord};

    fn result(passed: bool, output: &[&str]) -> CheckResult {
        CheckResult {
            commit: "abc".to_owned(),
            mode: "pr".to_owned(),
            passed,
            steps: vec![
                StepRecord {
                    name: "fmt".to_owned(),
                    passed: true,
                    duration_ms: 10,
                },
                StepRecord {
                    name: "clippy".to_owned(),
                    passed,
                    duration_ms: 20,
                },
            ],
            output: output.iter().map(|line| (*line).to_owned()).collect(),
        }
    }

    #[test]
    fn passing_result_has_a_summary_block_and_no_output_block() {
        let text = markdown(&result(true, &[]));
        assert_eq!(
            text,
            "## Check result\n\n```text\ncommit: abc\nmode: pr\nresult: pass\nsteps:\n  fmt: pass 10 ms\n  clippy: pass 20 ms\n```\n"
        );
    }

    #[test]
    fn failing_result_names_the_failing_step_and_fences_its_tail() {
        let text = markdown(&result(false, &["warning: unused"]));
        assert!(text.contains("  clippy: fail 20 ms\n"));
        assert!(
            text.contains(
                "Output of the failing step `clippy`:\n\n```text\nwarning: unused\n```\n"
            )
        );
    }

    #[test]
    fn fence_is_longer_than_any_backtick_run_in_the_output() {
        let text = fence(&["```rust".to_owned()]);
        assert!(text.starts_with("````text\n"));
        assert!(text.ends_with("\n````\n"));
    }

    #[test]
    fn failure_without_a_failed_step_uses_a_generic_heading() {
        let mut failed = result(false, &["cannot read the diff"]);
        failed.steps.clear();
        assert!(markdown(&failed).contains("Output:\n\n```text\ncannot read the diff\n```\n"));
    }
}
