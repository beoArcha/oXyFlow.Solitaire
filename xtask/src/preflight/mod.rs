//! `preflight [--card <ID>] [--base <ref>] [--ci]`: the last check before a pull request. It fetches
//! `origin`, reports how far the branch is behind and ahead of the base (warning when behind), runs
//! the conflict-marker and decision-ID lints over the tracked files and, with `--card`, warns about
//! changed files that the card's `Files` line does not name. Only lint errors and an unknown card
//! stop it. `--ci` then runs `check --pr <base>`, the gate that `pr.yml` runs.

mod card;
mod git;
mod glob;
#[cfg(test)]
mod tests;

use std::io;
use std::path::{Path, PathBuf};

use crate::check::{self, Mode};
use crate::error::XtaskError;
use crate::lint::{self, Severity};
use card::Card;

const DEFAULT_BASE: &str = "origin/main";

#[derive(Debug, thiserror::Error)]
pub enum PreflightError {
    #[error("cannot start `git`")]
    Spawn(#[source] io::Error),
    #[error("`git {command}` failed (exit code {code:?}): {stderr}")]
    Git {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("`git {command}` printed text that is not UTF-8")]
    NotUtf8 { command: String },
    #[error("`git {command}` did not print two commit counts")]
    Counts { command: String },
    #[error("no card `{id}` in `docs/task_list/*.md`")]
    UnknownCard { id: String },
    #[error("cannot read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub card: Option<String>,
    pub base: String,
    pub ci: bool,
}

/// Reads the flags after `preflight`; `None` when they do not fit. A flag with a value needs the value.
pub fn parse_options(flags: &[String]) -> Option<Options> {
    let mut options = Options {
        card: None,
        base: DEFAULT_BASE.to_owned(),
        ci: false,
    };
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        match flag.as_str() {
            "--ci" => options.ci = true,
            "--card" => options.card = Some(rest.next()?.clone()),
            "--base" => options.base = rest.next()?.clone(),
            _ => return None,
        }
    }
    Some(options)
}

pub fn run(root: &Path, options: &Options) -> Result<(), XtaskError> {
    let card = options
        .card
        .as_deref()
        .map(|id| card::find(root, id))
        .transpose()?;
    fetch_origin(root)?;
    report_divergence(root, &options.base)?;
    if let Some(card) = &card {
        report_card(card, &git::changed_files(root, &options.base)?);
    }
    let errors = report_lint(root)?;
    if errors > 0 {
        return Err(XtaskError::LintFailed { count: errors });
    }
    if options.ci {
        check::run(Mode::Pr(options.base.clone()))
    } else {
        Ok(())
    }
}

fn fetch_origin(root: &Path) -> Result<(), PreflightError> {
    let output = git::run(root, &["fetch", "origin"])?;
    if output.status.success() {
        println!("fetched origin");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let reason = stderr.trim().lines().next().unwrap_or_default();
        println!("NOTE fetch skipped, `git fetch origin` failed (offline?): {reason}");
    }
    Ok(())
}

fn report_divergence(root: &Path, base: &str) -> Result<(), PreflightError> {
    let (behind, ahead) = git::divergence(root, base)?;
    println!("branch is {behind} commit(s) behind and {ahead} commit(s) ahead of {base}");
    if behind > 0 {
        println!(
            "WARNING preflight behind: {behind} commit(s) of {base} are missing here; run `git merge {base}`"
        );
    }
    Ok(())
}

fn report_card(card: &Card, changed: &[String]) {
    println!("card {} names: {}", card.id, card.patterns.join(", "));
    for path in changed.iter().filter(|path| !card.names(path)) {
        println!("WARNING preflight card {} does not name {path}", card.id);
    }
}

fn report_lint(root: &Path) -> Result<usize, XtaskError> {
    let findings = lint::text_findings(root)?;
    for finding in &findings {
        lint::print_finding(finding);
    }
    Ok(findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count())
}
