//! The `docs` command: what a documentation-only pull request runs instead of the full build.
//! The conflict-marker and decision-ID lints, then a relative-link check over `docs/**/*.md`,
//! `README.md` and `AGENTS.md`. Anchors are not checked.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::error::XtaskError;
use crate::lint::{self, Finding, Severity};

const RULE: &str = "links";
const ROOT_FILES: [&str; 2] = ["README.md", "AGENTS.md"];

pub fn run(root: &Path) -> Result<(), XtaskError> {
    let mut findings = lint::text_findings(root)?;
    for relative in markdown_files(root)? {
        let path = root.join(&relative);
        let text = fs::read_to_string(&path).map_err(|source| XtaskError::Io { path, source })?;
        findings.extend(broken_links(&relative, &text, |target| {
            root.join(target).exists()
        }));
    }
    for finding in &findings {
        lint::print_finding(finding);
    }
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    if errors == 0 {
        Ok(())
    } else {
        Err(XtaskError::LintFailed { count: errors })
    }
}

fn markdown_files(root: &Path) -> Result<Vec<PathBuf>, XtaskError> {
    let output = Command::new("git")
        .args(["ls-files", "-z", "--", "docs", "README.md", "AGENTS.md"])
        .current_dir(root)
        .output()
        .map_err(|source| XtaskError::Spawn {
            program: "git".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(XtaskError::StepFailed {
            step: "docs",
            code: output.status.code(),
        });
    }
    let files = output
        .stdout
        .split(|byte| *byte == 0)
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .filter(|name| is_checked(name))
        .map(PathBuf::from)
        .filter(|path| root.join(path).is_file())
        .collect();
    Ok(files)
}

fn is_checked(name: &str) -> bool {
    ROOT_FILES.contains(&name) || (name.starts_with("docs/") && name.ends_with(".md"))
}

/// Findings for every relative link or image target in `text` that `exists` does not know.
/// `file` is the repository-relative path of the Markdown file; targets resolve against its folder.
pub fn broken_links(file: &Path, text: &str, exists: impl Fn(&Path) -> bool) -> Vec<Finding> {
    let folder = file.parent().unwrap_or(Path::new(""));
    let mut findings = Vec::new();
    for (line, target) in link_targets(text) {
        let Some(path) = local_path(&target) else {
            continue;
        };
        if !exists(&normalize(&folder.join(&path))) {
            findings.push(Finding {
                rule: RULE,
                severity: Severity::Error,
                file: file.to_path_buf(),
                line,
                message: format!("broken relative link `{target}`"),
            });
        }
    }
    findings
}

fn link_targets(text: &str) -> Vec<(usize, String)> {
    let mut targets = Vec::new();
    let mut fenced = false;
    for (index, line) in text.lines().enumerate() {
        let start = line.trim_start();
        if start.starts_with("```") || start.starts_with("~~~") {
            fenced = !fenced;
        } else if !fenced {
            let stripped = strip_code_spans(line);
            targets.extend(inline_targets(&stripped).map(|target| (index + 1, target)));
            targets.extend(definition_target(&stripped).map(|target| (index + 1, target)));
        }
    }
    targets
}

fn strip_code_spans(line: &str) -> String {
    line.split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 0)
        .map(|(_, part)| part)
        .collect()
}

fn inline_targets(line: &str) -> impl Iterator<Item = String> + '_ {
    line.match_indices("](").filter_map(|(at, _)| {
        let rest = &line[at + 2..];
        let end = rest.find(')')?;
        Some(clean_target(&rest[..end]))
    })
}

fn definition_target(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (_, after) = rest.split_once("]:")?;
    Some(clean_target(after))
}

fn clean_target(raw: &str) -> String {
    let trimmed = raw.trim();
    let target = match trimmed.strip_prefix('<') {
        Some(inner) => inner.split('>').next().unwrap_or(inner),
        None => trimmed.split_whitespace().next().unwrap_or(""),
    };
    target.to_owned()
}

fn local_path(target: &str) -> Option<PathBuf> {
    if target.is_empty() || target.starts_with('#') || target.starts_with('/') {
        return None;
    }
    if target.contains("{{") || has_scheme(target) {
        return None;
    }
    let path = target.split(['#', '?']).next().unwrap_or("");
    (!path.is_empty()).then(|| PathBuf::from(path.replace("%20", " ")))
}

fn has_scheme(target: &str) -> bool {
    target.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

fn normalize(path: &Path) -> PathBuf {
    let mut parts: Vec<Component> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir if matches!(parts.last(), Some(Component::Normal(_))) => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.iter().collect()
}

#[cfg(test)]
#[path = "docs/tests.rs"]
mod tests;
