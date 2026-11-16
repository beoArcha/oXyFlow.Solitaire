//! Accessibility regression tests: every action has a key and a help line, every interactive
//! element is reachable from the keyboard, every rejection and board event has an announcement in
//! both locales, and every theme meets its contrast target. The board checks are in `board.rs`.
//! The tests live inside the crate because they drive `App` through the test harness.

mod board;

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::KeyCode;

use crate::action::{Action, HelpGroup};
use crate::app::{Modal, Screen};
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale, t};
use crate::input::{binding_for, primary_key};
use crate::settings::GlyphMode;
use crate::test_support::Harness;
use crate::theme::contrast::contrast_ratio;
use crate::theme::{TEXT_PAIRS, ThemeId};

const MAX_ROWS: usize = 32;

const REJECTIONS: [&str; 13] = [
    "board.error.unsupported",
    "board.error.no_such_pile",
    "board.error.empty_source",
    "board.error.face_down",
    "board.error.invalid_sequence",
    "board.error.target_rejects",
    "board.error.stock_empty",
    "board.error.empty_column_blocks_deal",
    "board.error.passes_exhausted",
    "board.error.count_out_of_range",
    "board.error.not_enough_space",
    "board.error.not_a_pair",
    "board.error.blocked",
];

const EVENTS: [&str; 6] = [
    "board.status.moved",
    "board.status.dealt",
    "board.status.removed",
    "board.status.undone",
    "board.status.redone",
    "board.status.restarted",
];

fn focused(h: &mut Harness) -> Option<String> {
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    h.lines().iter().find_map(|line| {
        let after = line.split(glyphs.focus_left).nth(1)?;
        Some(after.split(glyphs.focus_right).next()?.trim().to_owned())
    })
}

/// The labels of a list in the order the arrow keys visit them; the list wraps, so the walk ends
/// where it began.
fn rows(h: &mut Harness) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    while labels.len() < MAX_ROWS {
        let Some(label) = focused(h) else { break };
        if labels.contains(&label) {
            break;
        }
        labels.push(label);
        h.press(KeyCode::Down);
    }
    labels
}

fn menu_at(index: usize) -> Harness {
    let mut h = Harness::new();
    for _ in 0..index {
        h.press(KeyCode::Down);
    }
    h
}

#[test]
fn every_action_has_a_key_and_a_help_line() {
    for action in Action::ALL {
        let binding = binding_for(action);
        assert!(
            binding.is_some_and(|b| !b.keys.is_empty()),
            "{action:?} has no key"
        );
        assert!(primary_key(action).is_some(), "{action:?} has no key name");
        assert!(
            binding.is_some_and(|b| HelpGroup::ALL.contains(&b.group)),
            "{action:?} is in no help group"
        );
        assert_ne!(t(action.label_key()), action.label_key(), "{action:?}");
    }
    for group in HelpGroup::ALL {
        assert_ne!(t(group.title_key()), group.title_key(), "{group:?}");
    }
}

#[test]
fn every_main_menu_item_is_reached_and_runs_by_keyboard() {
    let labels = rows(&mut Harness::new());
    assert_eq!(labels.len(), 7, "{labels:?}");
    for index in 0..labels.len() {
        let mut h = menu_at(index);
        h.press(KeyCode::Enter);
        match index {
            0 => assert_eq!(h.app.screen(), Screen::GameSelect),
            1 => {
                h.type_char('4');
                h.type_char('2');
                assert!(h.text().contains("42"), "seed field takes digits");
            }
            2 => assert_eq!(h.app.screen(), Screen::Stats),
            3 => assert_eq!(h.app.screen(), Screen::Settings),
            4 => assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_)))),
            5 => assert!(matches!(h.app.modal(), Some(Modal::Help))),
            _ => assert!(h.app.should_quit()),
        }
    }
}

#[test]
fn every_variant_is_reached_and_dealt_by_keyboard() {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    assert_eq!(rows(&mut h).len(), Variant::ALL.len());
    for (index, variant) in Variant::ALL.into_iter().enumerate() {
        let mut h = Harness::new();
        h.press(KeyCode::Enter);
        for _ in 0..index {
            h.press(KeyCode::Down);
        }
        h.press(KeyCode::Enter);
        assert_eq!(
            h.app.board().session().map(|s| s.variant()),
            Some(variant),
            "row {index}"
        );
    }
}

#[test]
fn a_seed_typed_by_keyboard_deals_the_chosen_variant() {
    let mut h = menu_at(1);
    h.press(KeyCode::Enter);
    for digit in "771".chars() {
        h.type_char(digit);
    }
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.board().session().map(|s| s.seed()), Some(771));
}

#[test]
fn every_settings_row_is_reached_by_keyboard() {
    let mut h = menu_at(3);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    let rows = rows(&mut h);
    assert_eq!(rows.len(), 11, "{rows:?}");
}

#[test]
fn every_dialog_closes_and_confirm_takes_yes_by_keyboard() {
    let mut h = Harness::new();
    h.app.apply(crate::screens::Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(3),
    });
    h.type_char('?');
    assert!(matches!(h.app.modal(), Some(Modal::Help)));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    h.type_char('i');
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    h.type_char('p');
    assert!(matches!(h.app.modal(), Some(Modal::Pause)));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    h.type_char('R');
    h.press(KeyCode::Right);
    assert!(matches!(h.app.modal(), Some(Modal::Confirm { .. })));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    h.type_char('R');
    h.press(KeyCode::Right);
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert!(h.text().contains("Game restarted."));
}

#[test]
fn every_theme_meets_its_contrast_target_for_the_text_pairs() {
    for id in ThemeId::ALL {
        let palette = id.palette();
        for (fg, bg) in TEXT_PAIRS {
            let ratio = contrast_ratio(palette.swatch(fg).rgb, palette.swatch(bg).rgb);
            assert!(
                ratio >= id.contrast_target(),
                "{id:?}: {fg:?} on {bg:?} is {ratio:.2}, target {}",
                id.contrast_target()
            );
        }
    }
    assert!((ThemeId::HighContrast.contrast_target() - 7.0).abs() < f32::EPSILON);
}

#[test]
fn every_rejection_and_board_event_is_announced_in_both_locales() {
    for locale in Locale::ALL {
        set_locale(locale);
        let prefix = match locale {
            Locale::EnUs => "Illegal move: ",
            Locale::PlPl => "Niedozwolony ruch: ",
        };
        for key in REJECTIONS {
            assert!(t(key).starts_with(prefix), "{locale:?} {key}: {}", t(key));
        }
        for key in EVENTS {
            assert_ne!(t(key), key, "{locale:?} {key}");
        }
    }
    set_locale(Locale::EnUs);
}
