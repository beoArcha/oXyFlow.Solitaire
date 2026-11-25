//! `cargo xtask`: development tooling for the oXySol workspace.

mod affected;
mod art;
mod check;
mod check_capture;
mod check_result;
#[cfg(feature = "art")]
mod curated_seeds;
mod docs;
mod error;
mod fonts;
#[cfg(feature = "art")]
mod icon;
mod lint;
#[cfg(feature = "art")]
mod locale_diff;
mod midi;
mod preflight;
mod report;
#[cfg(feature = "art")]
mod screenshots;
mod sync_metadata;
mod version;
#[allow(
    dead_code,
    reason = "the file is shared with the build scripts; xtask uses only the git-based part"
)]
mod version_scheme;
mod worktrees;

use std::env;
use std::error::Error;
use std::process::ExitCode;

use check::Mode;
use error::XtaskError;
use version::Output;

const USAGE: &str = "usage: cargo xtask <check [--quick | --pr <base>] | report | affected --base <ref> | lint | docs | fonts | midi <style> | sync-metadata [--check] | worktrees [--remove [--force]] | preflight [--card <ID>] [--base <ref>] [--ci] | version [--base | --tag]> (cargo xtask-art: icon [--check] | locale-diff [<lang>] [--ref <git-ref>] | screenshots | curated-seeds)";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            let mut cause = err.source();
            while let Some(source) = cause {
                eprintln!("  caused by: {source}");
                cause = source.source();
            }
            ExitCode::FAILURE
        }
    }
}

fn dispatch(args: &[String]) -> Result<(), XtaskError> {
    let Some((command, flags)) = args.split_first() else {
        return Err(usage());
    };
    match command.as_str() {
        "check" => check::run(parse_check(flags)?),
        "report" if flags.is_empty() => Ok(report::run(&check::workspace_root()?)?),
        "affected" => affected::run(&check::workspace_root()?, &parse_base(flags)?),
        "lint" if flags.is_empty() => lint::run(&check::workspace_root()?),
        "docs" if flags.is_empty() => docs::run(&check::workspace_root()?),
        art_command if art::COMMANDS.contains(&art_command) => art::run(art_command, flags),
        "fonts" if flags.is_empty() => Ok(fonts::run(&check::workspace_root()?)?),
        "midi" => Ok(midi::run(&check::workspace_root()?, &parse_midi(flags)?)?),
        "sync-metadata" => Ok(sync_metadata::run(parse_flag(flags, "--check")?)?),
        "worktrees" => Ok(worktrees::run(
            &check::workspace_root()?,
            worktrees::parse_options(flags).ok_or_else(usage)?,
        )?),
        "preflight" => preflight::run(
            &check::workspace_root()?,
            &preflight::parse_options(flags).ok_or_else(usage)?,
        ),
        "version" => version::run(parse_version_flags(flags)?),
        _ => Err(usage()),
    }
}

fn parse_version_flags(flags: &[String]) -> Result<Output, XtaskError> {
    match flags {
        [] => Ok(Output::Effective),
        [flag] if flag == "--base" => Ok(Output::Metadata),
        [flag] if flag == "--tag" => Ok(Output::Tag),
        _ => Err(usage()),
    }
}

fn parse_check(flags: &[String]) -> Result<Mode, XtaskError> {
    match flags {
        [] => Ok(Mode::Full),
        [flag] if flag == "--quick" => Ok(Mode::Quick),
        [flag, base] if flag == "--pr" => Ok(Mode::Pr(base.clone())),
        _ => Err(usage()),
    }
}

fn parse_midi(flags: &[String]) -> Result<String, XtaskError> {
    match flags {
        [style] => Ok(style.clone()),
        _ => Err(usage()),
    }
}

fn parse_base(flags: &[String]) -> Result<String, XtaskError> {
    match flags {
        [flag, base] if flag == "--base" => Ok(base.clone()),
        _ => Err(usage()),
    }
}

pub(crate) fn parse_flag(flags: &[String], allowed: &str) -> Result<bool, XtaskError> {
    let mut present = false;
    for flag in flags {
        if flag != allowed {
            return Err(usage());
        }
        present = true;
    }
    Ok(present)
}

pub(crate) fn usage() -> XtaskError {
    XtaskError::Usage(USAGE.to_owned())
}
