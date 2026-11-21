//! Screens and what they share with the application shell.
//!
//! A screen never touches the application directly. It receives a read-only [`ViewCtx`], and what
//! it wants done comes back as an [`Effect`] that `App` applies.

pub mod board;
pub mod focus;
pub mod game_select;
pub mod main_menu;
pub mod settings;
pub mod stats;

use std::time::{Duration, Instant};

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant, YukonRules};

use crate::action::Action;
use crate::app::{FontFacts, Modal, Screen, SearchCommand};
use crate::audio::{AudioStatus, StyleInfo};
use crate::caps::TerminalCaps;
use crate::glyphs::Glyphs;
use crate::settings::{Aids, CardSizeSetting, Settings};
use crate::theme::Theme;

/// The saved game as the main menu names it in its Resume row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedSummary {
    pub variant: Variant,
    pub elapsed: Duration,
    pub moves: usize,
}

/// Everything a screen may read while drawing or handling input.
pub struct ViewCtx<'a> {
    pub theme: &'a Theme,
    pub glyphs: &'a Glyphs,
    pub settings: &'a Settings,
    pub caps: &'a TerminalCaps,
    pub audio_status: AudioStatus,
    pub sound_styles: &'a [StyleInfo],
    /// The saved game, if any; `None` hides the Resume row.
    pub saved: Option<SavedSummary>,
    /// The last cell the mouse reported; a footer button under it is drawn as hovered.
    pub pointer: Option<(u16, u16)>,
    pub now: Instant,
    /// The terminal and the state of the oXySol console font (P5-02).
    pub font: FontFacts,
}

/// A request from a screen or a dialog to the application.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    None,
    Goto(Screen),
    /// Deal `variant`; `None` asks the application for a fresh seed. `undo_allowed` is the undo
    /// mode of the new game.
    Start {
        variant: Variant,
        seed: Option<u64>,
        undo_allowed: bool,
    },
    /// A button of the search dialog for a winnable deal.
    SearchStep(SearchCommand),
    /// Restore the saved game.
    Resume,
    /// Deal the game that was just played again from its seed (a button of the win dialog).
    SameSeed,
    /// Deal the running game again from its seed, with an empty history.
    Restart,
    /// Replace the settings; the application applies and saves them.
    Apply(Settings),
    /// Show `Settings` at once without saving them: a change of the Interface section. It is kept
    /// or undone when the player leaves the settings screen.
    Preview(Settings),
    /// Leave the settings screen for `to`, keeping the previewed interface changes (`keep`) or
    /// undoing them.
    LeaveSettings {
        keep: bool,
        to: Screen,
    },
    Open(Modal),
    /// Set the card size and save it with the settings (the zoom keys of the board).
    CardSize(CardSizeSetting),
    /// Set the aids of the running game; the defaults in the settings stay as they are.
    Quick(Aids),
    /// Make the aids of the running game the defaults for new games, and save them.
    SaveAids,
    /// Run a board action as its key would, after a dialog closed (a button of the moves window).
    Board(Action),
    /// Open the "How to play" window on the game being played, or on Klondike.
    HowToPlay,
    /// Copy the seed to the clipboard and announce the result.
    CopySeed(u64),
    /// Ask the solver whether these games of the history were winnable; the answers are stored.
    CheckWinnable(Vec<stats::Target>),
    Quit,
}

/// Localization key of the name of a game variant.
pub const fn variant_label_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Klondike(DrawCount::One) => "common.variant.klondike_one",
        Variant::Klondike(DrawCount::Three) => "common.variant.klondike_three",
        Variant::Spider(SuitCount::One) => "common.variant.spider_one",
        Variant::Spider(SuitCount::Two) => "common.variant.spider_two",
        Variant::Spider(SuitCount::Four) => "common.variant.spider_four",
        Variant::FreeCell => "common.variant.free_cell",
        Variant::Pyramid(PyramidRules::Relaxed) => "common.variant.pyramid_relaxed",
        Variant::Pyramid(PyramidRules::Classic) => "common.variant.pyramid_classic",
        Variant::Yukon(YukonRules::Classic) => "common.variant.yukon_classic",
        Variant::Yukon(YukonRules::Russian) => "common.variant.yukon_russian",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::t;

    #[test]
    fn every_variant_has_a_distinct_localized_name() {
        let names: Vec<String> = Variant::ALL
            .iter()
            .map(|variant| t(variant_label_key(*variant)).into_owned())
            .collect();
        for (variant, name) in Variant::ALL.iter().zip(&names) {
            assert_ne!(name, variant_label_key(*variant));
        }
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), names.len());
    }
}
