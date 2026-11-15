//! The `check` command: runs the quality gates in order and stops at the first failing step.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::affected::{self, Affected, ROOT};
use crate::error::XtaskError;
use crate::{lint, sync_metadata};

pub enum Mode {
    Full,
    Quick,
    Pr(String),
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
    run_step(&cargo, &root, "fmt", &["fmt", "--all", "--", "--check"])?;
    run_step(
        &cargo,
        &root,
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
    println!("==> lint");
    lint::run(&root)?;
    match mode {
        Mode::Full => {
            run_step(&cargo, &root, "test", &["test", "--workspace"])?;
            println!("==> sync-metadata --check");
            sync_metadata::run(true)?;
        }
        Mode::Quick => run_quick_tests(&cargo, &root)?,
        Mode::Pr(base) => run_pr_tests(&cargo, &root, &base)?,
    }
    println!("check passed");
    Ok(())
}

fn run_quick_tests(cargo: &str, root: &Path) -> Result<(), XtaskError> {
    run_step(
        cargo,
        root,
        "test",
        &["test", "--workspace", "--exclude", ROOT, "--tests"],
    )?;
    run_step(
        cargo,
        root,
        "test",
        &[
            "test",
            "-p",
            ROOT,
            "--lib",
            "--bins",
            "--test",
            "integration",
        ],
    )
}

fn run_pr_tests(cargo: &str, root: &Path, base: &str) -> Result<(), XtaskError> {
    run_step(
        cargo,
        root,
        "test",
        &[
            "test",
            "--workspace",
            "--test",
            "architecture",
            "--test",
            "e2e",
        ],
    )?;
    let paths = affected::changed_paths(root, base)?;
    match affected::select(paths.iter().map(String::as_str)) {
        Affected::All => {
            println!("==> every package is affected by changes since {base}");
            run_quick_tests(cargo, root)
        }
        Affected::Packages(packages) => {
            println!("==> affected packages since {base}: {packages:?}");
            for package in packages {
                run_package_tests(cargo, root, package)?;
            }
            Ok(())
        }
    }
}

fn run_package_tests(cargo: &str, root: &Path, package: &str) -> Result<(), XtaskError> {
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
    run_step(cargo, root, "test", args)
}

fn run_step(cargo: &str, root: &Path, step: &'static str, args: &[&str]) -> Result<(), XtaskError> {
    println!("==> {step}: cargo {}", args.join(" "));
    let status = Command::new(cargo)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|source| XtaskError::Spawn {
            program: cargo.to_owned(),
            source,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(XtaskError::StepFailed {
            step,
            code: status.code(),
        })
    }
}
