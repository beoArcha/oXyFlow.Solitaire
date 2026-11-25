//! Accessibility regression tests: every action has a key and a help line, every interactive
//! element is reachable from the keyboard, every rejection and board event has an announcement in
//! both locales, and every theme meets its contrast target. The board checks are in `board.rs`.
//! The tests live inside the crate because they drive `App` through the test harness.

mod board;
mod focus;
mod footer;
mod footer_hover;
mod footer_labels;

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::KeyCode;

use crate::action::{Action, HelpGroup};
use crate::app::{Modal, Screen};
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale, t};
use crate::input::{binding_for, primary_key};
use crate::screens::focus::marked_text;
use crate::screens::settings::options::FIELDS;
use crate::screens::settings::tabs::Tab;
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

/// Rows of the main menu's language selector, whose arrows are not a focus marker of a list.
const MENU_CORNER_ROWS: usize = 3;

fn focused(h: &mut Harness) -> Option<String> {
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    let lines = h.lines_without_pane();
    if h.app.screen() == Screen::Settings {
        return lines.into_iter().find_map(|line| {
            line.trim_start()
                .strip_prefix(glyphs.focus_left)
                .map(|after| after.trim().to_owned())
        });
    }
    let skip = if h.app.screen() == Screen::MainMenu {
        MENU_CORNER_ROWS
    } else {
        0
    };
    marked_text(lines.get(skip..).unwrap_or_default(), glyphs.focus_left)
}

/// The labels of a list in the order `key` visits them; the list wraps, so the walk ends where it
/// began.
fn walk(h: &mut Harness, key: KeyCode) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    while labels.len() < MAX_ROWS {
        let Some(label) = focused(h) else { break };
        if labels.contains(&label) {
            break;
        }
        labels.push(label);
        h.press(key);
    }
    labels
}

fn rows(h: &mut Harness) -> Vec<String> {
    walk(h, KeyCode::Down)
}

fn rows_by_tab(h: &mut Harness) -> Vec<String> {
    walk(h, KeyCode::Tab)
}

fn menu_at(index: usize) -> Harness {
    let mut h = Harness::new();
    for _ in 0..index {
        h.press(KeyCode::Down);
    }
    h
}

/// The position of a main menu item, by the localization key of its label.
fn menu_index(key: &str) -> usize {
    let label = t(key).into_owned();
    rows(&mut Harness::new())
        .iter()
        .position(|row| *row == label)
        .unwrap_or_default()
}

/// Every main menu item that a new game without a saved game shows, by the localization key of its
/// label. Each needs an expected outcome in `check_menu_outcome`.
const MENU_KEYS: [&str; 7] = [
    "common.menu.new_game",
    "menu.new_game_from_seed",
    "stats.label",
    "common.menu.settings",
    "menu.how_to_play.label",
    "menu.help",
    "common.menu.quit",
];

fn check_menu_outcome(key: &str, h: &mut Harness) {
    match key {
        "common.menu.new_game" => assert_eq!(h.app.screen(), Screen::GameSelect),
        "menu.new_game_from_seed" => assert!(
            h.text().contains(&t("menu.seed.choose_game").into_owned()),
            "the seed item opens the game list"
        ),
        "stats.label" => assert_eq!(h.app.screen(), Screen::Stats),
        "common.menu.settings" => assert_eq!(h.app.screen(), Screen::Settings),
        "menu.how_to_play.label" => assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_)))),
        "menu.help" => assert!(matches!(h.app.modal(), Some(Modal::Help(_)))),
        "common.menu.quit" => assert!(h.app.should_quit()),
        other => panic!("{other} has no expected outcome"),
    }
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
    for key in MENU_KEYS {
        let label = t(key).into_owned();
        assert!(labels.contains(&label), "{key} is not reached: {labels:?}");
    }
    for (index, label) in labels.iter().enumerate() {
        let Some(key) = MENU_KEYS
            .into_iter()
            .find(|key| t(key).as_ref() == label.as_str())
        else {
            panic!("{label:?} has no key in MENU_KEYS");
        };
        let mut h = menu_at(index);
        h.press(KeyCode::Enter);
        check_menu_outcome(key, &mut h);
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
    let mut h = menu_at(menu_index("menu.new_game_from_seed"));
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
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
    let mut h = menu_at(menu_index("common.menu.settings"));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    let mut canonical_rows = 0;
    for tab in [Tab::Popular, Tab::Gameplay, Tab::AudioVisual] {
        let tab_rows = rows(&mut h);
        assert_eq!(tab_rows.len(), tab.fields().len(), "{tab:?}: {tab_rows:?}");
        if tab != Tab::Popular {
            canonical_rows += tab_rows.len();
        }
        h.press(KeyCode::Tab);
    }
    assert_eq!(canonical_rows, FIELDS.len());
}

#[test]
fn every_main_menu_item_is_reached_by_tab_and_settings_tabs_switch_by_tab() {
    assert_eq!(rows_by_tab(&mut Harness::new()), rows(&mut Harness::new()));
    let mut h = menu_at(menu_index("common.menu.settings"));
    h.press(KeyCode::Enter);
    assert!(
        h.text()
            .contains(&format!("[{}]", t(Tab::Popular.label_key())))
    );
    h.press(KeyCode::Tab);
    assert!(
        h.text()
            .contains(&format!("[{}]", t(Tab::Gameplay.label_key())))
    );
    h.press(KeyCode::Tab);
    assert!(
        h.text()
            .contains(&format!("[{}]", t(Tab::AudioVisual.label_key())))
    );
    h.press(KeyCode::Tab);
    assert!(
        h.text()
            .contains(&format!("[{}]", t(Tab::Popular.label_key())))
    );
}

#[test]
fn every_dialog_closes_and_confirm_takes_yes_by_keyboard() {
    let mut h = Harness::new();
    h.app.apply(crate::screens::Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(3),
        undo_allowed: true,
    });
    h.type_char('?');
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    h.type_char('i');
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    h.type_char('p');
    assert!(matches!(h.app.modal(), Some(Modal::Pause { .. })));
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
    for locale in Locale::offered() {
        set_locale(locale);
        let prefix = t(REJECTIONS[0])
            .split_once(':')
            .map(|(head, _)| format!("{head}: "))
            .unwrap_or_default();
        for key in REJECTIONS {
            assert!(t(key).starts_with(&prefix), "{locale:?} {key}: {}", t(key));
        }
        for key in EVENTS.iter().chain(&REJECTIONS) {
            assert_ne!(t(key), *key, "{locale:?} {key}");
            assert!(
                t(key).chars().count() <= 40,
                "{locale:?} {key} is too long for the Now row: {}",
                t(key)
            );
        }
    }
    set_locale(Locale::EnUs);
}
