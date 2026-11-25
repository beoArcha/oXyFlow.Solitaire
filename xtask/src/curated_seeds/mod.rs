//! `cargo xtask-art curated-seeds`: generates the curated seed table of `oxysol_core::curated`
//! and its report, by the rules of decision T-49. Run it in a release build; it solves up to 5,000
//! deals per variant.

mod render;
mod scan;
mod score;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use oxysol_core::Variant;
use thiserror::Error;

const TABLE_PATH: &str = "crates/core/src/curated/table.rs";
const REPORT_PATH: &str = "docs/tech/curated-seeds.md";

/// Error while generating the curated seeds.
#[derive(Debug, Error)]
pub enum CuratedSeedsError {
    #[error("`{variant}` has {found} winnable seed(s) up to seed {last}; {needed} are needed")]
    TooFewWinnable {
        variant: String,
        found: usize,
        last: u64,
        needed: usize,
    },
    #[error("seed {seed} of `{variant}` does not replay to a win")]
    NotReplayable { variant: String, seed: u64 },
    #[error("the hint of `{variant}` seed {seed} is not a legal move")]
    IllegalHint { variant: String, seed: u64 },
    #[error("the scan of `{variant}` panicked")]
    ScanPanicked { variant: String },
    #[error("cannot write `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Scans every variant, then writes the table and the report under `root`.
pub fn run(root: &Path) -> Result<(), CuratedSeedsError> {
    let started = Instant::now();
    let scans = scan_all()?;
    let elapsed = started.elapsed();
    for scan in &scans {
        println!("{}", render::summary_line(scan));
    }
    write(root.join(TABLE_PATH), &render::table(&scans))?;
    write(root.join(REPORT_PATH), &render::report(&scans, elapsed))?;
    println!(
        "wrote {TABLE_PATH} and {REPORT_PATH} in {} s",
        elapsed.as_secs()
    );
    Ok(())
}

fn scan_all() -> Result<Vec<scan::VariantScan>, CuratedSeedsError> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = Variant::ALL
            .into_iter()
            .map(|variant| (variant, scope.spawn(move || scan::variant(variant))))
            .collect();
        handles
            .into_iter()
            .map(|(variant, handle)| match handle.join() {
                Ok(result) => result,
                Err(_) => Err(CuratedSeedsError::ScanPanicked {
                    variant: score::name(variant),
                }),
            })
            .collect()
    })
}

fn write(path: PathBuf, text: &str) -> Result<(), CuratedSeedsError> {
    fs::write(&path, text).map_err(|source| CuratedSeedsError::Write { path, source })
}

/// The wall-clock time of one run, written into the report as information only.
pub(super) fn run_time_text(elapsed: Duration) -> String {
    format!("{} s", elapsed.as_secs())
}
