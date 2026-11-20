//! The services `App` takes from the outside world.

use super::display::DisplayDeps;
use crate::audio::{Audio, StyleInfo};
use crate::caps::TerminalCaps;
use crate::clipboard::Clipboard;
use crate::solver::Solver;
use crate::storage::Storage;

/// Everything the application needs from the outside world.
pub struct Deps {
    pub storage: Box<dyn Storage>,
    pub audio: Box<dyn Audio>,
    pub clipboard: Box<dyn Clipboard>,
    pub caps: TerminalCaps,
    /// Fresh seed for "New game"; a typed seed bypasses it.
    pub seeds: Box<dyn FnMut() -> u64>,
    /// A seed from the command line: the main menu opens on the games dealt from it.
    pub start_seed: Option<u64>,
    /// The sound styles the settings offer, in order; the binary embeds them.
    pub sound_styles: Vec<StyleInfo>,
    /// Checks deals off the UI thread: the search for a winnable deal and the check of a typed seed.
    pub solver: Box<dyn Solver>,
    /// The console switch for full screen and the facts the decision needs.
    pub display: DisplayDeps,
    /// The effective version (T-21, R-23) the main menu shows; the binary computes it at build time.
    pub version: &'static str,
}
