//! The `check` command: runs the quality gates in order and stops at the first failing step.
//! Every run writes `target/check-result.txt` (see `check_result`), pass or fail.

use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::affected::{self, Affected, ROOT};
use crate::check_capture;
use crate::check_result::{self, StepLog};
use crate::error::XtaskError;
use crate::{icon, lint, sync_metadata};

pub enum Mode {
    Full,
    Quick,
    Pr(String),
}

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Quick => "quick",
            Self::Pr(_) => "pr",
        }
    }
}

pub fn workspace_root() -> Result<PathBuf, XtaskError> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or(XtaskError::WorkspaceRoot)
}

pub fn run(mode: Mode) -> Result<(), XtaskError> {
    let root = workspace_root()?;
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut runner = Runner {
        cargo: &cargo,
        root: &root,
        log: StepLog::default(),
    };
    let outcome = runner.gates(&mode);
    let failure = outcome.as_ref().err().map(ToString::to_string);
    let written = check_result::commit_id(&root).and_then(|commit| {
        let result = runner.log.into_result(commit, mode.label(), failure);
        check_result::save(&root, &result)
    });
    match (outcome, written) {
        (Ok(()), Ok(())) => {
            println!("check passed");
            Ok(())
        }
        (Err(err), Ok(())) => Err(err),
        (Ok(()), Err(err)) => Err(err.into()),
        (Err(err), Err(write)) => {
            eprintln!("error: cannot write the check result file: {write}");
            Err(err)
        }
    }
}

struct Runner<'a> {
    cargo: &'a str,
    root: &'a Path,
    log: StepLog,
}

impl Runner<'_> {
    fn gates(&mut self, mode: &Mode) -> Result<(), XtaskError> {
        let root = self.root;
        self.run_cargo("fmt", &["fmt", "--all", "--", "--check"])?;
        self.run_cargo(
            "clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        )?;
        self.in_process("lint", || {
            println!("==> lint");
            lint::run(root)
        })?;
        match mode {
            Mode::Full => {
                self.run_cargo("test", &["test", "--workspace"])?;
                self.in_process("sync-metadata", || {
                    println!("==> sync-metadata --check");
                    Ok(sync_metadata::run(true)?)
                })?;
                self.in_process("icon", || {
                    println!("==> icon --check");
                    Ok(icon::run(root, true)?)
                })?;
                Ok(())
            }
            Mode::Quick => self.quick_tests(),
            Mode::Pr(base) => self.pr_tests(base),
        }
    }

    fn quick_tests(&mut self) -> Result<(), XtaskError> {
        self.run_cargo("test", &["test", "--workspace", "--tests"])
    }

    fn ci_only_tests(&mut self) -> Result<(), XtaskError> {
        self.run_cargo(
            "test",
            &[
                "test",
                "--workspace",
                "--test",
                "architecture",
                "--test",
                "e2e",
            ],
        )
    }

    fn pr_tests(&mut self, base: &str) -> Result<(), XtaskError> {
        let paths = affected::changed_paths(self.root, base)?;
        match affected::select(paths.iter().map(String::as_str)) {
            Affected::All => {
                println!("==> every package is affected by changes since {base}");
                self.quick_tests()
            }
            Affected::Packages(packages) => {
                println!("==> affected packages since {base}: {packages:?}");
                self.ci_only_tests()?;
                for package in packages {
                    self.package_tests(package)?;
                }
                Ok(())
            }
        }
    }

    fn package_tests(&mut self, package: &str) -> Result<(), XtaskError> {
        let args: &[&str] = if package == ROOT {
            &[
                "test",
                "-p",
                package,
                "--lib",
                "--bins",
                "--test",
                "integration",
            ]
        } else {
            &["test", "-p", package, "--tests"]
        };
        self.run_cargo("test", args)
    }

    fn run_cargo(&mut self, step: &'static str, args: &[&str]) -> Result<(), XtaskError> {
        println!("==> {step}: cargo {}", args.join(" "));
        let started = Instant::now();
        let captured = check_capture::run_cargo(self.cargo, self.root, args)?;
        self.log
            .record(step, captured.passed, started.elapsed(), captured.tail);
        if captured.passed {
            Ok(())
        } else {
            Err(XtaskError::StepFailed {
                step,
                code: captured.code,
            })
        }
    }

    fn in_process(
        &mut self,
        step: &'static str,
        body: impl FnOnce() -> Result<(), XtaskError>,
    ) -> Result<(), XtaskError> {
        let started = Instant::now();
        let outcome = body();
        self.log
            .record(step, outcome.is_ok(), started.elapsed(), Vec::new());
        outcome
    }
}
