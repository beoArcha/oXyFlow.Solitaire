//! `cargo xtask`: development tooling for the oXySol workspace.

mod affected;
mod check;
mod error;
mod icon;
mod lint;
mod midi;
mod sync_metadata;
mod version;

use std::env;
use std::error::Error;
use std::process::ExitCode;

use check::Mode;
use error::XtaskError;
use version::Output;

const USAGE: &str = "usage: cargo xtask <check [--quick | --pr <base>] | affected --base <ref> | lint | midi <style> | icon [--check] | sync-metadata [--check] | version [--tag [--stable]]>";

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
        "affected" => affected::run(&check::workspace_root()?, &parse_base(flags)?),
        "lint" if flags.is_empty() => lint::run(&check::workspace_root()?),
        "midi" => Ok(midi::run(&check::workspace_root()?, &parse_midi(flags)?)?),
        "icon" => Ok(icon::run(
            &check::workspace_root()?,
            parse_flag(flags, "--check")?,
        )?),
        "sync-metadata" => Ok(sync_metadata::run(parse_flag(flags, "--check")?)?),
        "version" => version::run(parse_version_flags(flags)?),
        _ => Err(usage()),
    }
}

fn parse_version_flags(flags: &[String]) -> Result<Output, XtaskError> {
    match flags {
        [] => Ok(Output::Version),
        [flag] if flag == "--tag" => Ok(Output::Tag { stable: false }),
        [flag, stable] if flag == "--tag" && stable == "--stable" => {
            Ok(Output::Tag { stable: true })
        }
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

fn parse_flag(flags: &[String], allowed: &str) -> Result<bool, XtaskError> {
    let mut present = false;
    for flag in flags {
        if flag != allowed {
            return Err(usage());
        }
        present = true;
    }
    Ok(present)
}

fn usage() -> XtaskError {
    XtaskError::Usage(USAGE.to_owned())
}
