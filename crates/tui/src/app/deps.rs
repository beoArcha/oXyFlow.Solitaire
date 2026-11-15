//! The services `App` takes from the outside world.

use crate::audio::Audio;
use crate::caps::TerminalCaps;
use crate::clipboard::Clipboard;
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
}
