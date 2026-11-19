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

use std::time::Instant;

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};

use crate::app::{Modal, Screen, SearchCommand};
use crate::audio::{AudioStatus, StyleInfo};
use crate::caps::TerminalCaps;
use crate::glyphs::Glyphs;
use crate::settings::Settings;
use crate::theme::Theme;

/// Everything a screen may read while drawing or handling input.
pub struct ViewCtx<'a> {
    pub theme: &'a Theme,
    pub glyphs: &'a Glyphs,
    pub settings: &'a Settings,
    pub caps: &'a TerminalCaps,
    pub audio_status: AudioStatus,
    pub sound_styles: &'a [StyleInfo],
    pub has_saved_game: bool,
    pub now: Instant,
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
    /// Deal the running game again from its seed, with an empty history.
    Restart,
    /// Replace the settings; the application applies and saves them.
    Apply(Settings),
    Open(Modal),
    /// Open the "How to play" window on the game being played, or on Klondike.
    HowToPlay,
    /// Copy the seed to the clipboard and announce the result.
    CopySeed(u64),
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
