//! The `Files` line of a task card in `docs/task_list/*.md`.

use std::fs;
use std::path::{Path, PathBuf};

use super::PreflightError;
use super::glob;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub patterns: Vec<String>,
}

impl Card {
    pub fn names(&self, path: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| glob::matches(pattern, path))
    }
}

/// The card whose heading is `### <id> —`, from the first task list file that has it.
pub fn find(root: &Path, id: &str) -> Result<Card, PreflightError> {
    for path in task_lists(root)? {
        let text = fs::read_to_string(&path).map_err(|source| PreflightError::Io {
            path: path.clone(),
            source,
        })?;
        if let Some(card) = parse(&text, id) {
            return Ok(card);
        }
    }
    Err(PreflightError::UnknownCard { id: id.to_owned() })
}

fn task_lists(root: &Path) -> Result<Vec<PathBuf>, PreflightError> {
    let dir = root.join("docs").join("task_list");
    let io_error = |source| PreflightError::Io {
        path: dir.clone(),
        source,
    };
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dir).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

pub fn parse(text: &str, id: &str) -> Option<Card> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|line| is_heading(line, id))?;
    let patterns = lines[start + 1..]
        .iter()
        .take_while(|line| !line.starts_with('#'))
        .filter_map(|line| files_value(line))
        .flat_map(backticked)
        .collect();
    Some(Card {
        id: id.to_owned(),
        patterns,
    })
}

fn is_heading(line: &str, id: &str) -> bool {
    line.strip_prefix("### ")
        .and_then(|rest| rest.strip_prefix(id))
        .is_some_and(|rest| rest.trim_start().starts_with('—'))
}

/// The text after `**Files:**` or `**Shared files:**` on a card bullet such as `- **Files:** ...`.
fn files_value(line: &str) -> Option<&str> {
    let text = line.trim_start().trim_start_matches('-').trim_start();
    text.strip_prefix("**Files:**")
        .or_else(|| text.strip_prefix("**Shared files:**"))
}

fn backticked(value: &str) -> Vec<String> {
    value
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::trim)
        .filter(|span| !span.is_empty())
        .map(str::to_owned)
        .collect()
}
