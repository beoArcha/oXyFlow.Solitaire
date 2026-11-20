//! Rule `conflict-markers`: git conflict markers left in any tracked text file.
//!
//! `<<<<<<<` and `>>>>>>>` lines (exactly seven characters, then nothing or a space and a label)
//! are flagged anywhere. A `=======` line is flagged only inside an open `<<<<<<<` block, so a
//! Markdown setext heading underline is never flagged.

use super::{Finding, Severity, SourceFile};

const RULE: &str = "conflict-markers";
const WIDTH: usize = 7;

pub fn check(file: &SourceFile) -> Vec<Finding> {
    let mut in_conflict = false;
    let mut findings = Vec::new();
    for (index, line) in file.text.lines().enumerate() {
        let flagged = match marker(line) {
            Some('<') => {
                in_conflict = true;
                true
            }
            Some('>') => {
                in_conflict = false;
                true
            }
            Some('=') => in_conflict,
            _ => false,
        };
        if flagged {
            findings.push(Finding {
                rule: RULE,
                severity: Severity::Error,
                file: file.path.clone(),
                line: index + 1,
                message: "leftover git conflict marker".to_owned(),
            });
        }
    }
    findings
}

fn marker(line: &str) -> Option<char> {
    let first = line.chars().next()?;
    if !matches!(first, '<' | '=' | '>') {
        return None;
    }
    let run = first.to_string().repeat(WIDTH);
    let rest = line.strip_prefix(run.as_str())?;
    let valid = match first {
        '=' => rest.trim().is_empty(),
        _ => rest.is_empty() || rest.starts_with(' '),
    };
    valid.then_some(first)
}

#[cfg(test)]
#[path = "conflict_markers/tests.rs"]
mod tests;
