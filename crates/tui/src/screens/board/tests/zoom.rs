//! Zoom on the board: the keys `]`, `[` and `0`, Ctrl+wheel, the setting that remembers the size, the
//! fallback when a size does not fit and the limits the status names.

use oxysol_core::{DrawCount, PyramidRules, Variant};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use super::*;
use crate::input::Input;
use crate::settings::{CardSizeSetting, Settings};
use crate::theme::ThemeId;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const PYRAMID: Variant = Variant::Pyramid(PyramidRules::Relaxed);

fn board(variant: Variant, size: (u16, u16), settings: Settings) -> Harness {
    let storage = MemoryStorage {
        settings: Some(settings),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, size.0, size.1);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(7),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn auto(variant: Variant, size: (u16, u16)) -> Harness {
    board(variant, size, Settings::default())
}

fn card(h: &mut Harness) -> (u16, u16) {
    h.lines();
    let board = h.app.board();
    let geometry = board.geometry(&board.piles());
    geometry.map_or((0, 0), |geometry| {
        (geometry.card.width, geometry.card.height)
    })
}

fn saved(h: &Harness) -> Option<CardSizeSetting> {
    h.storage
        .borrow()
        .settings
        .as_ref()
        .map(|settings| settings.card_size)
}

fn wheel(h: &mut Harness, kind: MouseEventKind, modifiers: KeyModifiers) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column: 60,
        row: 20,
        modifiers,
    }));
}

#[test]
fn auto_is_the_default_and_shows_the_largest_size_that_fits() {
    assert_eq!(Settings::default().card_size, CardSizeSetting::Auto);
    assert_eq!(card(&mut auto(KLONDIKE, (120, 40))), (11, 9));
    assert_eq!(card(&mut auto(KLONDIKE, (200, 60))), (13, 11));
    assert_eq!(card(&mut auto(KLONDIKE, (100, 30))), (7, 5));
}

#[test]
fn the_keys_step_down_and_up_and_the_limits_say_so() {
    let mut h = auto(KLONDIKE, (120, 40));
    h.type_char('[');
    assert_eq!(card(&mut h), (9, 7));
    assert_eq!(saved(&h), Some(CardSizeSetting::Medium));
    h.type_char('[');
    assert_eq!(card(&mut h), (7, 5));
    h.type_char('[');
    assert_eq!(card(&mut h), (7, 5));
    assert!(h.text().contains("Smallest size"));
    h.type_char(']');
    h.type_char(']');
    assert_eq!(card(&mut h), (11, 9));
    assert_eq!(saved(&h), Some(CardSizeSetting::Large));
    h.type_char(']');
    assert_eq!(card(&mut h), (11, 9));
    assert!(h.text().contains("Largest size that fits"));
}

#[test]
fn zero_returns_to_auto_and_says_so() {
    let mut h = auto(KLONDIKE, (200, 60));
    h.type_char('[');
    h.type_char('[');
    assert_eq!(card(&mut h), (9, 7));
    h.type_char('0');
    assert_eq!(card(&mut h), (13, 11));
    assert_eq!(saved(&h), Some(CardSizeSetting::Auto));
    assert!(h.text().contains("Card size: auto"));
}

#[test]
fn a_chosen_size_that_does_not_fit_falls_back_and_the_game_information_says_auto() {
    let settings = Settings {
        card_size: CardSizeSetting::Huge,
        ..Settings::default()
    };
    let mut h = board(KLONDIKE, (120, 40), settings);
    assert_eq!(card(&mut h), (11, 9));
    assert!(h.text().contains("Cards: Large (auto)"));
    let mut fits = board(
        KLONDIKE,
        (120, 40),
        Settings {
            card_size: CardSizeSetting::Medium,
            ..Settings::default()
        },
    );
    assert_eq!(card(&mut fits), (9, 7));
    let text = fits.text();
    assert!(text.contains("Cards: Medium"));
    assert!(!text.contains("(auto)"));
    assert!(!auto(KLONDIKE, (120, 40)).text().contains("Cards:"));
}

#[test]
fn zooming_in_from_a_fallback_has_nothing_larger_to_show() {
    let settings = Settings {
        card_size: CardSizeSetting::Huge,
        ..Settings::default()
    };
    let mut h = board(KLONDIKE, (120, 40), settings);
    h.type_char(']');
    assert_eq!(card(&mut h), (11, 9));
    assert!(h.text().contains("Largest size that fits"));
    h.type_char('[');
    assert_eq!(card(&mut h), (9, 7));
}

#[test]
fn the_remembered_size_is_used_by_the_next_game() {
    let mut h = auto(KLONDIKE, (120, 40));
    h.type_char('[');
    let remembered = h.storage.borrow().settings.clone().unwrap_or_default();
    let mut next = board(KLONDIKE, (120, 40), remembered);
    assert_eq!(card(&mut next), (9, 7));
}

#[test]
fn the_field_frame_of_a_theme_counts_when_the_largest_size_is_decided() {
    let framed = Settings {
        theme: ThemeId::HighContrast,
        ..Settings::default()
    };
    let mut dark = auto(KLONDIKE, (120, 38));
    let mut contrast = board(KLONDIKE, (120, 38), framed);
    assert_eq!(card(&mut dark), (11, 9));
    assert_eq!(card(&mut contrast), (9, 7));
    dark.type_char(']');
    assert!(dark.text().contains("Largest size that fits"));
    contrast.type_char(']');
    assert_eq!(card(&mut contrast), (9, 7));
    assert!(contrast.text().contains("Largest size that fits"));
}

#[test]
fn the_pyramid_zooms_too() {
    let mut h = auto(PYRAMID, (120, 40));
    assert_eq!(card(&mut h), (9, 7));
    h.type_char('[');
    assert_eq!(card(&mut h), (7, 5));
    h.type_char('0');
    assert_eq!(card(&mut h), (9, 7));
}

#[test]
fn ctrl_wheel_zooms_and_a_plain_wheel_does_not() {
    let mut h = auto(KLONDIKE, (120, 40));
    wheel(&mut h, MouseEventKind::ScrollDown, KeyModifiers::NONE);
    assert_eq!(card(&mut h), (11, 9));
    wheel(&mut h, MouseEventKind::ScrollDown, KeyModifiers::CONTROL);
    assert_eq!(card(&mut h), (9, 7));
    wheel(&mut h, MouseEventKind::ScrollUp, KeyModifiers::CONTROL);
    assert_eq!(card(&mut h), (11, 9));
}

#[test]
fn the_zoom_keys_leave_the_run_size_keys_alone() {
    let mut h = start(KLONDIKE, 7);
    h.lines();
    let before = h.app.board().cursor;
    h.type_char('+');
    h.type_char('-');
    assert_eq!(h.app.board().cursor, before);
    assert_eq!(h.app.settings().card_size, CardSizeSetting::Auto);
    h.send(Input::Key(KeyEvent::new(
        KeyCode::Char('+'),
        KeyModifiers::NONE,
    )));
    assert_eq!(h.app.settings().card_size, CardSizeSetting::Auto);
}

#[test]
fn the_quick_choices_are_not_saved_by_a_zoom() {
    let mut h = auto(KLONDIKE, (120, 40));
    h.type_char('g');
    h.type_char('[');
    let stored = h.storage.borrow().settings.clone().unwrap_or_default();
    assert!(stored.guide_panel);
}

#[test]
fn the_polish_game_says_the_same_in_polish() {
    let settings = Settings {
        locale: crate::i18n::Locale::PlPl,
        ..Settings::default()
    };
    let mut h = board(KLONDIKE, (120, 40), settings);
    h.type_char('[');
    h.type_char('[');
    h.type_char('[');
    assert!(h.text().contains("Najmniejszy rozmiar"));
    assert!(h.text().contains("Karty: Małe"));
    h.type_char('0');
    assert!(h.text().contains("Rozmiar kart: auto"));
    crate::i18n::set_locale(crate::i18n::Locale::EnUs);
}

#[test]
fn the_guide_panel_lists_the_zoom_keys_in_both_locales() {
    for (locale, label) in [
        (crate::i18n::Locale::EnUs, "card size"),
        (crate::i18n::Locale::PlPl, "rozmiar kart"),
    ] {
        let settings = Settings {
            locale,
            ..Settings::default()
        };
        let mut h = board(KLONDIKE, (200, 60), settings);
        let text = h.text();
        assert!(text.contains("]/[/0"), "{text}");
        assert!(text.contains(label), "{label}");
    }
    crate::i18n::set_locale(crate::i18n::Locale::EnUs);
}

#[test]
fn the_help_window_lists_the_zoom_keys_and_the_terminal_zoom() {
    let mut h = auto(KLONDIKE, (120, 40));
    h.type_char('?');
    let text = h.text();
    for needle in [
        "Display",
        "Bigger cards",
        "Smaller cards",
        "Cards: auto size",
    ] {
        assert!(text.contains(needle), "{needle}");
    }
    assert!(text.contains("Ctrl +/-/0"));
    assert!(text.contains("Terminal zoom"));
}
