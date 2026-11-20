//! The look of the aura: style, size and colour (P3-29), seen in the buffer of a started game.

use oxysol_core::{DrawCount, Variant};

use super::aura::painted;
use super::*;
use crate::settings::{AuraColour, AuraSize, AuraStyle, GlyphMode, Settings};
use crate::theme::{Theme, ThemeId, Token};

fn started(settings: Settings) -> Harness {
    let storage = MemoryStorage {
        settings: Some(settings),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(5),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn with_look(style: AuraStyle, size: AuraSize) -> Settings {
    Settings {
        aura_style: style,
        aura_size: size,
        ..Settings::default()
    }
}

#[test]
fn a_thick_halo_covers_more_cells_than_a_thin_one_and_keeps_them() {
    let mut thin = started(with_look(AuraStyle::Halo, AuraSize::Thin));
    let mut thick = started(with_look(AuraStyle::Halo, AuraSize::Thick));
    let thin_cells = painted(&mut thin, "░", Token::AuraFocus);
    let thick_cells = painted(&mut thick, "░", Token::AuraFocus);
    assert!(!thin_cells.is_empty());
    assert!(thick_cells.len() > thin_cells.len());
    assert!(thin_cells.is_subset(&thick_cells));
}

#[test]
fn the_frame_style_draws_lines_instead_of_shades() {
    let mut h = started(with_look(AuraStyle::Frame, AuraSize::Thin));
    assert!(painted(&mut h, "░", Token::AuraFocus).is_empty());
    for glyph in ["─", "│"] {
        assert!(
            !painted(&mut h, glyph, Token::AuraFocus).is_empty(),
            "{glyph}"
        );
    }
    let corners = ["┌", "┐", "└", "┘"]
        .into_iter()
        .filter(|corner| !painted(&mut h, corner, Token::AuraFocus).is_empty())
        .count();
    assert!(corners > 0);
}

#[test]
fn a_thick_frame_draws_two_lines() {
    let mut thin = started(with_look(AuraStyle::Frame, AuraSize::Thin));
    let mut thick = started(with_look(AuraStyle::Frame, AuraSize::Thick));
    let thin_cells = painted(&mut thin, "│", Token::AuraFocus);
    let thick_cells = painted(&mut thick, "│", Token::AuraFocus);
    assert!(thick_cells.len() > thin_cells.len());
}

#[test]
fn the_frame_is_plain_ascii_in_ascii_mode() {
    let settings = Settings {
        glyph_mode: GlyphMode::Ascii,
        ..with_look(AuraStyle::Frame, AuraSize::Thin)
    };
    let mut h = started(settings);
    for glyph in ["-", "|", "+"] {
        assert!(
            !painted(&mut h, glyph, Token::AuraFocus).is_empty(),
            "{glyph}"
        );
    }
    assert!(h.text().is_ascii());
}

#[test]
fn marks_only_draws_no_halo() {
    let mut h = started(with_look(AuraStyle::MarksOnly, AuraSize::Thick));
    for glyph in ["░", "▓", "·", "┌", "│"] {
        assert!(
            painted(&mut h, glyph, Token::AuraFocus).is_empty(),
            "{glyph}"
        );
    }
}

#[test]
fn a_chosen_colour_replaces_the_colour_of_the_halo() {
    let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
    assert_ne!(theme.color(Token::AuraFocus), theme.color(Token::Hint));
    let settings = Settings {
        aura_colour: AuraColour::Hint,
        ..Settings::default()
    };
    let mut h = started(settings);
    assert!(!painted(&mut h, "░", Token::Hint).is_empty());
    assert!(painted(&mut h, "░", Token::AuraFocus).is_empty());
}
