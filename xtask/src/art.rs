//! The commands that need the application crates and the SVG renderer (`icon`, `locale-diff`,
//! `screenshots`, `curated-seeds`). They sit behind the `art` feature so the other commands build without
//! `oxysol-tui`, `oxysol-core`, `ratatui` and `resvg`; run them with `cargo xtask-art`.

use crate::error::XtaskError;

pub const COMMANDS: [&str; 4] = ["icon", "locale-diff", "screenshots", "curated-seeds"];

#[cfg(not(feature = "art"))]
pub fn run(command: &str, _flags: &[String]) -> Result<(), XtaskError> {
    let command = COMMANDS
        .iter()
        .copied()
        .find(|known| *known == command)
        .unwrap_or("icon");
    Err(XtaskError::ArtRequired { command })
}

#[cfg(feature = "art")]
pub fn run(command: &str, flags: &[String]) -> Result<(), XtaskError> {
    use crate::check::workspace_root;
    use crate::{curated_seeds, icon, locale_diff, screenshots};

    let root = workspace_root()?;
    match command {
        "icon" => Ok(icon::run(&root, crate::parse_flag(flags, "--check")?)?),
        "screenshots" if flags.is_empty() => Ok(screenshots::run(&root)?),
        "curated-seeds" if flags.is_empty() => Ok(curated_seeds::run(&root)?),
        "locale-diff" => {
            let options = locale_diff::parse_options(flags).ok_or_else(crate::usage)?;
            Ok(locale_diff::run(&root, &options)?)
        }
        _ => Err(crate::usage()),
    }
}
