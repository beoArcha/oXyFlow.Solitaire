//! The preview pane of the settings screen (O-102): it follows every look setting and fits the
//! window beside the list, under it or not at all.

use ratatui::crossterm::event::KeyCode;

use super::super::options::{FIELDS, Field};
use super::{index_of, move_down, open_settings};
use crate::i18n::{Locale, set_locale, t};
use crate::test_support::Harness;

const LABELS: [&str; 4] = [
    "settings.pane.back",
    "settings.pane.focus",
    "settings.pane.carried",
    "settings.pane.target",
];

const PREVIEWED: [Field; 12] = [
    Field::Theme,
    Field::CardBack,
    Field::CardFace,
    Field::FrontStyle,
    Field::BackStyle,
    Field::Glyphs,
    Field::CardSize,
    Field::AuraColour,
    Field::AuraStyle,
    Field::AuraSize,
    Field::Aura,
    Field::TargetMarks,
];

fn side_by_side() -> Harness {
    let mut h = open_settings();
    h.resize(130, 40);
    h
}

fn focus(h: &mut Harness, field: Field) {
    move_down(h, index_of(field));
}

#[test]
fn every_look_setting_changes_the_preview() {
    for field in PREVIEWED {
        let mut h = side_by_side();
        focus(&mut h, field);
        let before = h.pane_cells();
        assert!(!before.is_empty(), "{field:?}: no preview pane");
        h.press(KeyCode::Right);
        assert_ne!(h.pane_cells(), before, "{field:?}");
    }
}

#[test]
fn the_card_size_steps_each_change_the_preview() {
    let mut h = side_by_side();
    focus(&mut h, Field::CardSize);
    let mut seen = vec![h.pane_cells()];
    for _ in 0..3 {
        h.press(KeyCode::Right);
        seen.push(h.pane_cells());
    }
    for (index, cells) in seen.iter().enumerate() {
        assert!(
            !seen[..index].contains(cells),
            "card size step {index} looks like an earlier one"
        );
    }
}

#[test]
fn a_setting_outside_the_look_leaves_the_preview_alone() {
    let mut h = side_by_side();
    focus(&mut h, Field::Volume);
    let before = h.pane_cells();
    h.press(KeyCode::Left);
    assert_eq!(h.pane_cells(), before);
}

#[test]
fn the_card_back_row_no_longer_carries_a_swatch() {
    let mut h = side_by_side();
    let line = super::line_of(&mut h, &t(Field::CardBack.label_key()));
    let value = line.trim_end().trim_end_matches(['►', '◄']).trim_end();
    assert!(value.ends_with(&*t("settings.card_back.theme")), "{value}");
}

#[test]
fn the_preview_shows_the_cards_in_the_chosen_glyphs() {
    let mut h = side_by_side();
    let text = h.text();
    for card in ["Q♥", "J♠", "K♦"] {
        assert!(text.contains(card), "{card}");
    }
    focus(&mut h, Field::Glyphs);
    h.press(KeyCode::Right);
    let text = h.text();
    assert!(!text.contains('♥') && !text.contains('♠'), "{text}");
}

#[test]
fn the_preview_stands_beside_the_list_in_a_wide_window_in_every_language() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = side_by_side();
        for (index, field) in FIELDS.iter().enumerate() {
            let list = h.lines_without_pane();
            let marked = format!("► {}", t(field.label_key()));
            assert!(
                list.iter().any(|line| line.contains(&marked)),
                "{locale:?} row {index}"
            );
            h.press(KeyCode::Down);
        }
        let text = h.text();
        assert!(text.contains(&*t("settings.pane.title")), "{locale:?}");
        for key in LABELS {
            assert!(text.contains(&*t(key)), "{locale:?} {key}");
        }
        assert!(!text.contains(&*t("settings.pane.hidden")), "{locale:?}");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_window_100_by_30_keeps_the_whole_list_and_says_the_preview_is_hidden() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = open_settings();
        h.resize(100, 30);
        let text = h.text();
        assert!(text.contains(&*t("settings.pane.hidden")), "{locale:?}");
        let title = t("settings.pane.title");
        assert!(text.lines().all(|line| line.trim() != title), "{locale:?}");
        assert!(text.lines().count() <= 30);
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_narrow_high_window_puts_the_preview_under_the_list() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = open_settings();
        h.resize(100, 52);
        focus(&mut h, Field::AutoFinish);
        let lines = h.lines();
        let title = lines
            .iter()
            .position(|line| line.contains(&*t("settings.pane.title")));
        let last_row = lines
            .iter()
            .rposition(|line| line.contains(&*t(Field::AutoFinish.label_key())));
        assert!(title.is_some() && last_row.is_some(), "{locale:?}");
        assert!(title > last_row, "{locale:?}");
        let text = lines.join("\n");
        for key in LABELS {
            assert!(text.contains(&*t(key)), "{locale:?} {key}");
        }
    }
    set_locale(Locale::EnUs);
}
