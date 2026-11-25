//! Interface look rows: preview at once, confirmation on leaving, and keeping that behaviour on
//! Popular aliases as well as in Audio & Visual.

use ratatui::crossterm::event::KeyCode;

use super::super::options::{FIELDS, Field, Step};
use super::super::tabs::Tab;
use super::{changed, focus_in_tab, is_console_font_row, open_settings, row, switch_to_tab, value};
use crate::app::{Modal, Screen};
use crate::i18n::{Locale, set_locale, t};
use crate::settings::{AuraColour, AuraSize, AuraStyle, CardBack, Settings};
use crate::test_support::Harness;
use crate::theme::ThemeId;

const AURA_ROWS: [Field; 3] = [Field::AuraColour, Field::AuraStyle, Field::AuraSize];

fn focus(h: &mut Harness, field: Field) {
    focus_in_tab(h, super::canonical_tab(field), field);
}

fn saved(h: &Harness) -> Option<Settings> {
    h.storage.borrow().settings.clone()
}

fn leave_and_answer(h: &mut Harness, keep: bool) {
    h.press(KeyCode::Esc);
    assert!(matches!(h.app.modal(), Some(Modal::Confirm { .. })));
    assert_eq!(h.app.screen(), Screen::Settings);
    if keep {
        h.press(KeyCode::Tab);
    }
    h.press(KeyCode::Enter);
}

#[test]
fn all_interface_rows_live_in_audio_and_visual_and_match_with_interface_of() {
    let av = Tab::AudioVisual.fields();
    for field in [
        Field::Theme,
        Field::ColorVision,
        Field::CardBack,
        Field::CardFace,
        Field::FourColorDeck,
        Field::TableColour,
        Field::TableFeel,
        Field::FrontStyle,
        Field::BackStyle,
        Field::Glyphs,
        Field::ConsoleFont,
        Field::FontFace,
        Field::FontSize,
        Field::CardSize,
        Field::AuraColour,
        Field::AuraStyle,
        Field::AuraSize,
        Field::Priority,
    ] {
        assert!(av.contains(&field), "{field:?}");
        assert!(field.is_interface(), "{field:?}");
    }
    let base = Settings::default();
    for field in FIELDS {
        if is_console_font_row(field) {
            continue;
        }
        let stepped = changed(field, &base, Step::Next);
        if stepped == base {
            continue;
        }
        let restored = stepped.with_interface_of(&base);
        if field.is_interface() {
            assert_eq!(
                restored, base,
                "{field:?} should be restored by with_interface_of"
            );
        } else {
            assert_ne!(
                restored, base,
                "{field:?} should not be in with_interface_of"
            );
        }
    }
}

#[test]
fn the_aura_rows_cycle_their_values_in_both_directions() {
    let base = Settings::default();
    let mut colour = base.clone();
    for expected in [
        AuraColour::Text,
        AuraColour::Selection,
        AuraColour::Hint,
        AuraColour::Theme,
    ] {
        colour = changed(Field::AuraColour, &colour, Step::Next);
        assert_eq!(colour.aura_colour, expected);
    }
    assert_eq!(
        changed(Field::AuraColour, &base, Step::Previous).aura_colour,
        AuraColour::Hint
    );
    let frame = changed(Field::AuraStyle, &base, Step::Next);
    assert_eq!(frame.aura_style, AuraStyle::Frame);
    let marks = changed(Field::AuraStyle, &frame, Step::Next);
    assert_eq!(marks.aura_style, AuraStyle::MarksOnly);
    assert_eq!(
        changed(Field::AuraStyle, &marks, Step::Next).aura_style,
        AuraStyle::Halo
    );
    assert_eq!(
        changed(Field::AuraStyle, &base, Step::Previous).aura_style,
        AuraStyle::MarksOnly
    );
    let thick = changed(Field::AuraSize, &base, Step::Next);
    assert_eq!(thick.aura_size, AuraSize::Thick);
    assert_eq!(
        changed(Field::AuraSize, &thick, Step::Next).aura_size,
        AuraSize::Thin
    );
}

#[test]
fn the_aura_rows_name_their_values_in_english_and_polish() {
    let english = [
        ("Aura color", "Theme", "Halo", "Thin"),
        ("Aura style", "Text color", "Frame", "Thick"),
        ("Aura size", "Selection color", "Marks only", "Thin"),
    ];
    set_locale(Locale::EnUs);
    let base = Settings::default();
    assert!(row(Field::AuraColour, &base, true).starts_with(english[0].0));
    assert!(row(Field::AuraStyle, &base, true).starts_with(english[1].0));
    assert!(row(Field::AuraSize, &base, true).starts_with(english[2].0));
    assert_eq!(value(Field::AuraColour, &base), "Theme");
    assert_eq!(value(Field::AuraStyle, &base), "Halo");
    assert_eq!(value(Field::AuraSize, &base), "Thin");
    set_locale(Locale::PlPl);
    assert!(row(Field::AuraColour, &base, true).starts_with("Kolor aury"));
    assert_eq!(value(Field::AuraStyle, &base), "Halo");
    assert_eq!(value(Field::AuraSize, &base), "Cienka");
    set_locale(Locale::EnUs);
}

#[test]
fn every_interface_row_previews_at_once_without_saving() {
    for field in Tab::AudioVisual.fields() {
        if !field.is_interface() || is_console_font_row(*field) || *field == Field::Glyphs {
            continue;
        }
        let mut h = open_settings();
        focus(&mut h, *field);
        let before = h.app.settings().clone();
        h.press(KeyCode::Right);
        assert_ne!(h.app.settings(), &before, "{field:?}");
        assert_eq!(saved(&h), None, "{field:?}");
    }
}

#[test]
fn popular_interface_aliases_keep_preview_and_confirm() {
    for field in [Field::Theme, Field::CardSize, Field::CardBack] {
        let mut h = open_settings();
        focus_in_tab(&mut h, Tab::Popular, field);
        let before = h.app.settings().clone();
        h.press(KeyCode::Right);
        assert_ne!(h.app.settings(), &before, "{field:?}");
        assert_eq!(saved(&h), None, "{field:?} preview not saved yet");
        assert!(h.text().contains(&*t("settings.preview.note")), "{field:?}");
        leave_and_answer(&mut h, false);
        assert_eq!(h.app.settings(), &before, "{field:?} restored");
        assert!(saved(&h).is_none_or(|s| s == before), "{field:?}");
    }
}

#[test]
fn keeping_the_preview_saves_it_and_leaves_the_screen() {
    for field in AURA_ROWS {
        let mut h = open_settings();
        focus(&mut h, field);
        let before = h.app.settings().clone();
        h.press(KeyCode::Right);
        let previewed = h.app.settings().clone();
        leave_and_answer(&mut h, true);
        assert_eq!(h.app.screen(), Screen::MainMenu, "{field:?}");
        assert!(h.app.modal().is_none(), "{field:?}");
        assert_eq!(h.app.settings(), &previewed, "{field:?}");
        assert_eq!(saved(&h), Some(previewed), "{field:?}");
        assert_ne!(h.app.settings(), &before);
    }
}

#[test]
fn undoing_the_preview_restores_the_settings_and_saves_nothing_new() {
    for field in AURA_ROWS {
        let mut h = open_settings();
        focus(&mut h, field);
        let before = h.app.settings().clone();
        h.press(KeyCode::Right);
        leave_and_answer(&mut h, false);
        assert_eq!(h.app.screen(), Screen::MainMenu, "{field:?}");
        assert_eq!(h.app.settings(), &before, "{field:?}");
        assert_ne!(saved(&h).map(|s| s.aura_style), Some(AuraStyle::Frame));
        assert!(saved(&h).is_none_or(|s| s == before), "{field:?}");
    }
}

#[test]
fn escape_in_the_question_undoes_the_preview() {
    let mut h = open_settings();
    focus(&mut h, Field::Theme);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().theme, ThemeId::Light);
    h.press(KeyCode::Esc);
    assert!(matches!(h.app.modal(), Some(Modal::Confirm { .. })));
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert_eq!(h.app.settings().theme, ThemeId::Dark);
}

#[test]
fn a_preview_that_returns_to_where_it_started_asks_nothing() {
    let mut h = open_settings();
    focus(&mut h, Field::AuraSize);
    h.press(KeyCode::Right);
    h.press(KeyCode::Left);
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn a_row_outside_the_section_saves_without_the_preview() {
    let mut h = open_settings();
    focus(&mut h, Field::CardBack);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().card_back, CardBack::Monokai);
    focus(&mut h, Field::Cues);
    h.press(KeyCode::Enter);
    let written = saved(&h);
    assert!(written.as_ref().is_some_and(|s| !s.cues));
    assert!(written.is_some_and(|s| s.card_back == CardBack::Theme));
    assert_eq!(h.app.settings().card_back, CardBack::Monokai);
}

#[test]
fn the_screen_says_a_preview_is_open_and_the_note_goes_with_it() {
    let mut h = open_settings();
    assert!(!h.text().contains(&*t("settings.preview.note")));
    focus(&mut h, Field::AuraStyle);
    h.press(KeyCode::Right);
    assert!(h.text().contains(&*t("settings.preview.note")));
    leave_and_answer(&mut h, false);
}

#[test]
fn a_click_on_the_arrow_of_an_interface_row_previews_too() {
    let mut h = open_settings();
    focus(&mut h, Field::AuraSize);
    let lines = h.lines_without_pane();
    let found = lines.iter().enumerate().find_map(|(row, line)| {
        if !line.trim_start().starts_with('►') {
            return None;
        }
        let column = line.chars().position(|c| c == '►')?;
        Some((u16::try_from(column).ok()?, u16::try_from(row).ok()?))
    });
    assert!(found.is_some());
    let (column, row) = found.unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.settings().aura_size, AuraSize::Thick);
    assert_eq!(saved(&h), None);
}

#[test]
fn every_row_stays_reachable_at_100_by_30_in_every_offered_language() {
    for locale in Locale::offered() {
        set_locale(locale);
        for tab in Tab::ALL {
            let mut h = open_settings();
            h.resize(100, 30);
            switch_to_tab(&mut h, tab);
            for (index, field) in tab.fields().iter().enumerate() {
                let marked = format!("► {}", t(field.label_key()));
                let text = h.text();
                assert!(
                    text.contains(&marked),
                    "{locale:?} {tab:?} row {index}:\n{text}"
                );
                assert!(text.lines().count() <= 30);
                h.press(KeyCode::Down);
            }
        }
    }
    set_locale(Locale::EnUs);
}
