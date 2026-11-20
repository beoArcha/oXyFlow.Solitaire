//! Rule `decision-ids`: decision IDs in `docs/tech/decisions.md` are unique, and every `R-`, `T-`
//! or `D-` reference in the Markdown under `docs/` and in `AGENTS.md` names a decision that exists.
//!
//! A decision is defined by a table row whose first cell is the ID (`| R-01 | ... |`), or by a
//! heading that starts with the ID (`### D-06 ...`), because some decisions are written as
//! sections. A repeated definition is an error. An unknown reference is a warning, so a typo does
//! not fail the check, and it is reported at its file and line. A reference is an ID with a
//! non-word character or the line edge on both sides, so `UTF-8` or `R-01x` is not one.

use std::collections::BTreeMap;
use std::path::Path;

use super::{Finding, Severity, SourceFile};

const RULE: &str = "decision-ids";
const DECISIONS: &str = "docs/tech/decisions.md";
const AGENTS: &str = "AGENTS.md";

pub fn check(files: &[SourceFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let known = known_ids(files, &mut findings);
    for file in files.iter().filter(|file| is_reference_source(&file.path)) {
        for (index, line) in file.text.lines().enumerate() {
            for id in references(line) {
                if !known.contains_key(id) {
                    findings.push(finding(
                        file,
                        index + 1,
                        Severity::Warning,
                        format!("reference to unknown decision {id}"),
                    ));
                }
            }
        }
    }
    findings
}

fn known_ids(files: &[SourceFile], findings: &mut Vec<Finding>) -> BTreeMap<String, usize> {
    let mut known = BTreeMap::new();
    let Some(decisions) = files
        .iter()
        .find(|file| file.path.as_path() == Path::new(DECISIONS))
    else {
        return known;
    };
    for (id, line) in definitions(&decisions.text) {
        match known.get(&id).copied() {
            Some(first) => findings.push(finding(
                decisions,
                line,
                Severity::Error,
                format!("duplicate decision ID {id}, first defined at line {first}"),
            )),
            None => {
                known.insert(id, line);
            }
        }
    }
    known
}

fn finding(file: &SourceFile, line: usize, severity: Severity, message: String) -> Finding {
    Finding {
        rule: RULE,
        severity,
        file: file.path.clone(),
        line,
        message,
    }
}

fn is_reference_source(path: &Path) -> bool {
    path == Path::new(AGENTS)
        || (path.starts_with("docs") && path.extension().is_some_and(|ext| ext == "md"))
}

fn definitions(text: &str) -> Vec<(String, usize)> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| definition_id(line).map(|id| (id.to_owned(), index + 1)))
        .collect()
}

fn definition_id(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let id = match line.strip_prefix('|') {
        Some(row) => row.split('|').next()?.trim(),
        None => {
            let heading = line.strip_prefix('#')?.trim_start_matches('#');
            heading.strip_prefix(' ')?.split_whitespace().next()?
        }
    };
    is_id(id).then_some(id)
}

fn is_id(text: &str) -> bool {
    let bytes = text.as_bytes();
    matches!(bytes.first(), Some(b'R' | b'T' | b'D'))
        && bytes.get(1) == Some(&b'-')
        && bytes.len() > 2
        && bytes[2..].iter().all(u8::is_ascii_digit)
}

fn references(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    for start in 0..bytes.len() {
        if !is_reference_start(bytes, start) {
            continue;
        }
        let digits = bytes[start + 2..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        let end = start + 2 + digits;
        let bounded = !bytes.get(end).is_some_and(|byte| is_word(*byte));
        if digits > 0 && bounded {
            found.push(&line[start..end]);
        }
    }
    found
}

fn is_reference_start(bytes: &[u8], start: usize) -> bool {
    matches!(bytes[start], b'R' | b'T' | b'D')
        && bytes.get(start + 1) == Some(&b'-')
        && !(start > 0 && is_word(bytes[start - 1]))
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
#[path = "decision_ids/tests.rs"]
mod tests;
