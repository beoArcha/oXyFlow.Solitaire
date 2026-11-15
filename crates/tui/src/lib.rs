//! Terminal user interface of oXySol. Renders the engine state and turns input into commands.

pub mod action;
pub mod announce;
pub mod app;
pub mod audio;
pub mod caps;
pub mod clock;
pub mod glyphs;
pub mod i18n;
pub mod input;
pub mod layout;
pub mod mouse;
pub mod screens;
pub mod settings;
pub mod storage;
pub mod terminal;
pub mod theme;
pub mod widgets;

#[cfg(test)]
mod accessibility_tests;
#[cfg(test)]
mod test_support;

pub use app::{App, Deps, Modal, Screen};
pub use audio::{Audio, AudioSettings, NullAudio, SoundCue};
pub use input::Input;
/// The terminal event types of [`Input`], so that callers can build input without their own dependency.
pub use ratatui::crossterm;
pub use settings::Settings;
pub use storage::{GameId, Outcome, SavedGame, Storage, StorageError};
pub use terminal::{TuiError, restore_terminal, run};
