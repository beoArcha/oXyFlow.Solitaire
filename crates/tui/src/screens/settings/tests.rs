use ratatui::crossterm::event::KeyCode;

mod guide;
mod screen;

use super::options::{
    BACK_STYLES, FIELDS, FRONT_STYLES, Field, SOUND_STYLES, Step, changed, line, value_text,
};
use crate::app::{Modal, Screen};
use crate::i18n::{Locale, locale, set_locale, t};
use crate::settings::{ColorMode, GlyphMode, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use crate::theme::ThemeId;

/// Position of Settings in the main menu without a saved game.
const SETTINGS_ITEM: usize = 3;

fn open_settings() -> Harness {
    let mut h = Harness::new();
    move_down(&mut h, SETTINGS_ITEM);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h
}

fn move_down(h: &mut Harness, rows: usize) {
    for _ in 0..rows {
        h.press(KeyCode::Down);
    }
}

fn is_style(field: Field) -> bool {
    matches!(
        field,
        Field::FrontStyle | Field::BackStyle | Field::SoundStyle
    )
}

#[test]
fn every_row_with_several_choices_changes_the_settings_when_stepped() {
    let base = Settings::default();
    for field in FIELDS.into_iter().filter(|field| !is_style(*field)) {
        assert_ne!(changed(field, &base, Step::Next), base, "{field:?}");
    }
}

#[test]
fn style_rows_keep_their_only_choice_until_more_styles_exist() {
    let base = Settings::default();
    for field in [Field::FrontStyle, Field::BackStyle, Field::SoundStyle] {
        assert_eq!(changed(field, &base, Step::Next), base, "{field:?}");
        assert_eq!(changed(field, &base, Step::Previous), base, "{field:?}");
    }
}

#[test]
fn themes_and_languages_cycle_both_ways() {
    let base = Settings::default();
    let light = changed(Field::Theme, &base, Step::Next);
    assert_eq!(light.theme, ThemeId::Light);
    let high = changed(Field::Theme, &light, Step::Next);
    assert_eq!(high.theme, ThemeId::HighContrast);
    assert_eq!(
        changed(Field::Theme, &high, Step::Next).theme,
        ThemeId::Dark
    );
    assert_eq!(
        changed(Field::Theme, &base, Step::Previous).theme,
        ThemeId::HighContrast
    );
    let polish = changed(Field::Language, &base, Step::Next);
    assert_eq!(polish.locale, Locale::PlPl);
    assert_eq!(
        changed(Field::Language, &polish, Step::Next).locale,
        Locale::EnUs
    );
}

#[test]
fn glyph_and_color_modes_cycle() {
    let base = Settings::default();
    assert_eq!(base.glyph_mode, GlyphMode::Unicode);
    assert_eq!(
        changed(Field::Glyphs, &base, Step::Next).glyph_mode,
        GlyphMode::Ascii
    );
    assert_eq!(
        changed(Field::Glyphs, &base, Step::Previous).glyph_mode,
        GlyphMode::Font
    );
    assert_eq!(base.color_mode, ColorMode::Auto);
    assert_eq!(
        changed(Field::Colors, &base, Step::Next).color_mode,
        ColorMode::Ansi16
    );
    assert_eq!(
        changed(Field::Colors, &base, Step::Previous).color_mode,
        ColorMode::TrueColor
    );
}

#[test]
fn unknown_style_ids_move_to_the_first_offer() {
    let settings = Settings {
        front_style: "custom".to_owned(),
        back_style: "custom".to_owned(),
        sound_style: "custom".to_owned(),
        ..Settings::default()
    };
    assert_eq!(
        changed(Field::FrontStyle, &settings, Step::Next).front_style,
        FRONT_STYLES[0]
    );
    assert_eq!(
        changed(Field::BackStyle, &settings, Step::Next).back_style,
        BACK_STYLES[0]
    );
    assert_eq!(
        changed(Field::SoundStyle, &settings, Step::Next).sound_style,
        SOUND_STYLES[0]
    );
}

#[test]
fn the_default_styles_are_offered() {
    let base = Settings::default();
    assert!(FRONT_STYLES.contains(&base.front_style.as_str()));
    assert!(BACK_STYLES.contains(&base.back_style.as_str()));
    assert!(SOUND_STYLES.contains(&base.sound_style.as_str()));
}

#[test]
fn volume_moves_in_tens_and_stays_between_0_and_100() {
    let at = |volume: u8| Settings {
        volume,
        ..Settings::default()
    };
    assert_eq!(changed(Field::Volume, &at(70), Step::Next).volume, 80);
    assert_eq!(changed(Field::Volume, &at(70), Step::Previous).volume, 60);
    assert_eq!(changed(Field::Volume, &at(95), Step::Next).volume, 100);
    assert_eq!(changed(Field::Volume, &at(100), Step::Next).volume, 100);
    assert_eq!(changed(Field::Volume, &at(5), Step::Previous).volume, 0);
    assert_eq!(changed(Field::Volume, &at(0), Step::Previous).volume, 0);
}

#[test]
fn cues_and_music_toggle() {
    let base = Settings::default();
    assert!(!changed(Field::Cues, &base, Step::Next).cues);
    assert!(changed(Field::Music, &base, Step::Next).music);
}

#[test]
fn values_name_the_theme_locale_and_volume() {
    let base = Settings::default();
    assert_eq!(value_text(Field::Language, &base), "English");
    assert_eq!(value_text(Field::Theme, &base), "Dark");
    assert_eq!(value_text(Field::Volume, &base), "70%");
    assert_eq!(value_text(Field::Cues, &base), "On");
}

#[test]
fn unavailable_sound_rows_say_so_in_their_text() {
    let base = Settings::default();
    assert!(line(Field::Music, &base, false).ends_with("(unavailable)"));
    assert!(!line(Field::Music, &base, true).contains("unavailable"));
    assert!(!line(Field::Theme, &base, false).contains("unavailable"));
}

#[test]
fn every_row_changes_by_keyboard_and_reaches_storage() {
    for (index, field) in FIELDS.iter().enumerate() {
        let mut h = open_settings();
        move_down(&mut h, index);
        let before = h.app.settings().clone();
        h.press(KeyCode::Enter);
        let expected = changed(*field, &before, Step::Next);
        assert_eq!(h.app.settings(), &expected, "{field:?}");
        assert_eq!(
            h.storage.borrow().settings.clone(),
            Some(expected),
            "{field:?}"
        );
    }
}

#[test]
fn left_and_right_step_the_focused_row() {
    let mut h = open_settings();
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().locale, Locale::PlPl);
    assert_eq!(
        h.storage.borrow().settings.as_ref().map(|s| s.locale),
        Some(Locale::PlPl)
    );
    h.press(KeyCode::Left);
    assert_eq!(h.app.settings().locale, Locale::EnUs);
}

#[test]
fn volume_is_adjusted_with_the_arrows_and_clamped() {
    let mut h = open_settings();
    move_down(&mut h, FIELDS.len() - 2);
    for _ in 0..10 {
        h.press(KeyCode::Left);
    }
    assert_eq!(h.app.settings().volume, 0);
    for _ in 0..12 {
        h.press(KeyCode::Right);
    }
    assert_eq!(h.app.settings().volume, 100);
    assert!(h.text().contains("Volume: 100%"));
}

#[test]
fn a_theme_change_is_shown_at_once() {
    let mut h = open_settings();
    move_down(&mut h, 1);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().theme, ThemeId::Light);
    assert!(h.text().contains("Theme: Light"));
}

#[test]
fn a_language_change_switches_the_locale_at_once() {
    let mut h = open_settings();
    h.press(KeyCode::Right);
    assert_eq!(locale(), Locale::PlPl);
    let expected = format!(
        "{}: {}",
        t("settings.row.language"),
        t("settings.locale.pl_pl")
    );
    assert!(h.text().contains(&expected));
    set_locale(Locale::EnUs);
}

#[test]
fn escape_returns_to_the_main_menu() {
    let mut h = open_settings();
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn the_focused_row_is_marked_without_color() {
    let mut h = open_settings();
    assert!(h.text().contains("► Language:"));
    let mut ascii = Harness::with(MemoryStorage::new(), PLAIN, 120, 40);
    move_down(&mut ascii, SETTINGS_ITEM);
    ascii.press(KeyCode::Enter);
    let text = ascii.text();
    assert!(text.contains("> Language:"));
    assert!(text.is_ascii());
}

#[test]
fn a_storage_failure_alerts_and_the_setting_holds_for_the_session() {
    let failing = MemoryStorage {
        failing: true,
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(failing, TRUE_COLOR, 120, 40);
    assert_eq!(
        h.app.modal(),
        Some(&Modal::alert("common.error.settings_load"))
    );
    h.press(KeyCode::Enter);
    move_down(&mut h, SETTINGS_ITEM);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.settings().locale, Locale::PlPl);
    assert_eq!(h.app.modal(), Some(&Modal::alert("common.error.storage")));
    assert_eq!(h.app.screen(), Screen::Settings);
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.settings().theme, ThemeId::Light);
    assert_eq!(h.app.modal(), Some(&Modal::alert("common.error.storage")));
}

#[test]
fn the_row_text_uses_the_line_format() {
    let base = Settings::default();
    assert_eq!(line(Field::Volume, &base, true), "Volume: 70%");
    assert_eq!(t("settings.row.line"), "{label}: {value}");
}

#[test]
fn a_click_on_a_row_reaches_the_settings_through_the_app() {
    let mut h = open_settings();
    let found = h.find("Theme:");
    assert!(found.is_some());
    let (column, row) = found.unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.settings().theme, ThemeId::Light);
    assert_eq!(
        h.storage.borrow().settings.as_ref().map(|s| s.theme),
        Some(ThemeId::Light)
    );
}
