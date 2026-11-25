use ratatui::crossterm::event::KeyCode;

mod arrows;
mod card_back;
mod card_face;
mod card_size;
mod console_font;
mod focus;
mod font_rows;
mod footer;
mod groups;
mod guide;
mod hints;
mod interface;
mod language;
mod low_vision;
mod preview;
mod screen;
mod styles;
mod table;
mod tabs;
mod unavailable;
mod window_size;

use super::options::{FIELDS, Field, Step};
use super::tabs::Tab;
use super::values::{line, value_text};
use crate::app::{FontFacts, Modal, Screen};
use crate::i18n::{Locale, locale, set_locale, t};
use crate::settings::{AuraMode, ColorMode, FullScreen, GlyphMode, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR, sound_styles};
use crate::theme::ThemeId;

fn changed(field: Field, settings: &Settings, step: Step) -> Settings {
    super::values::changed(field, settings, step, &sound_styles())
}

fn row(field: Field, settings: &Settings, audio_available: bool) -> String {
    line(
        field,
        settings,
        audio_available,
        &sound_styles(),
        (200, FontFacts::default()),
        None,
    )
}

fn value(field: Field, settings: &Settings) -> String {
    value_text(field, settings, &sound_styles())
}

pub(super) fn line_of(h: &mut Harness, label: &str) -> String {
    h.lines_without_pane()
        .into_iter()
        .find(|line| line.contains(label))
        .map(|line| line.trim_end().to_owned())
        .unwrap_or_default()
}

/// A row's text without the arrows that frame the value of the focused row, so that its end is the
/// value.
pub(super) fn bare(line: &str) -> &str {
    line.trim_end().trim_end_matches(['►', '>']).trim_end()
}

/// The most main menu rows that are looked through for the Settings item.
const MAX_MENU_STEPS: usize = 32;

/// Position of Settings in the main menu without a saved game, found by its focus mark.
fn settings_menu_index() -> usize {
    let mut h = Harness::new();
    let mark = format!("► {}", t("common.menu.settings"));
    (0..MAX_MENU_STEPS)
        .find(|_| {
            let shown = h.lines().iter().any(|line| line.contains(&mark));
            if !shown {
                h.press(KeyCode::Down);
            }
            shown
        })
        .unwrap_or_default()
}

/// A window 130 columns wide and as high as the tallest tab needs with its chrome, so every row
/// of any tab is drawn.
pub(super) fn fitting_size() -> (u16, u16) {
    let rows = Tab::ALL
        .into_iter()
        .map(|tab| tab.fields().len())
        .max()
        .unwrap_or_default();
    let lines = u16::try_from(rows).unwrap_or(u16::MAX);
    (130, lines + super::CHROME_ROWS)
}

pub(super) fn canonical_tab(field: Field) -> Tab {
    if Tab::AudioVisual.fields().contains(&field) {
        Tab::AudioVisual
    } else {
        Tab::Gameplay
    }
}

pub(super) fn switch_to_tab(h: &mut Harness, target: Tab) {
    let expected = format!("[{}]", t(target.label_key()));
    for _ in 0..Tab::ALL.len() {
        if h.text().contains(&expected) {
            return;
        }
        h.press(KeyCode::Tab);
    }
}

pub(super) fn focus_in_tab(h: &mut Harness, tab: Tab, field: Field) {
    switch_to_tab(h, tab);
    let label = format!("► {} .", t(field.label_key()));
    for _ in 0..tab.fields().len() {
        if h.lines_without_pane()
            .iter()
            .any(|line| line.trim_start().starts_with(&label))
        {
            return;
        }
        h.press(KeyCode::Down);
    }
}

fn index_of(field: Field) -> usize {
    canonical_tab(field)
        .fields()
        .iter()
        .position(|row| *row == field)
        .unwrap_or_default()
}

fn open_settings() -> Harness {
    let mut h = Harness::new();
    move_down(&mut h, settings_menu_index());
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h
}

fn move_down(h: &mut Harness, rows: usize) {
    for _ in 0..rows {
        h.press(KeyCode::Down);
    }
}

/// The rows that only an owned classic Windows console lets the game change; the default test
/// terminal is not one.
pub(super) fn is_console_font_row(field: Field) -> bool {
    matches!(
        field,
        Field::ConsoleFont | Field::FontFace | Field::FontSize | Field::WindowSize
    )
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
fn themes_and_languages_cycle_both_ways() {
    let base = Settings::default();
    let light = changed(Field::Theme, &base, Step::Next);
    assert_eq!(light.theme, ThemeId::Light);
    let high = changed(Field::Theme, &light, Step::Next);
    assert_eq!(high.theme, ThemeId::HighContrast);
    let monokai = changed(Field::Theme, &high, Step::Next);
    assert_eq!(monokai.theme, ThemeId::Monokai);
    let dracula = changed(Field::Theme, &monokai, Step::Next);
    assert_eq!(dracula.theme, ThemeId::Dracula);
    assert_eq!(
        changed(Field::Theme, &dracula, Step::Next).theme,
        ThemeId::Dark
    );
    assert_eq!(
        changed(Field::Theme, &base, Step::Previous).theme,
        ThemeId::Dracula
    );
    let polish = changed(Field::Language, &base, Step::Next);
    assert_eq!(polish.locale, Locale::PlPl);
    assert_eq!(
        changed(Field::Language, &polish, Step::Previous).locale,
        Locale::EnUs
    );
    assert_eq!(
        Some(changed(Field::Language, &base, Step::Previous).locale),
        Locale::offered().last().copied()
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
    let ascii = changed(Field::Glyphs, &base, Step::Next);
    assert_eq!(
        changed(Field::Glyphs, &ascii, Step::Next).glyph_mode,
        GlyphMode::Font
    );
    let font = changed(Field::Glyphs, &ascii, Step::Next);
    assert_eq!(
        changed(Field::Glyphs, &font, Step::Next).glyph_mode,
        GlyphMode::Unicode
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
fn the_full_screen_row_cycles_auto_on_off_in_the_display_group() {
    let base = Settings::default();
    assert_eq!(base.full_screen, FullScreen::Auto);
    let on = changed(Field::FullScreen, &base, Step::Next);
    assert_eq!(on.full_screen, FullScreen::On);
    let off = changed(Field::FullScreen, &on, Step::Next);
    assert_eq!(off.full_screen, FullScreen::Off);
    assert_eq!(
        changed(Field::FullScreen, &off, Step::Next).full_screen,
        FullScreen::Auto
    );
    assert_eq!(
        changed(Field::FullScreen, &base, Step::Previous).full_screen,
        FullScreen::Off
    );
    assert_eq!(value(Field::FullScreen, &base), "Auto");
    assert_eq!(value(Field::FullScreen, &on), "On");
    assert!(row(Field::FullScreen, &off, true).starts_with("Full screen"));
    assert!(Tab::AudioVisual.fields().contains(&Field::FullScreen));
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
fn the_aura_row_cycles_between_full_and_focus_only() {
    let base = Settings::default();
    let focus = changed(Field::Aura, &base, Step::Next);
    assert_eq!(focus.aura, AuraMode::FocusOnly);
    assert_eq!(
        changed(Field::Aura, &focus, Step::Next).aura,
        AuraMode::Full
    );
    assert_eq!(value(Field::Aura, &focus), "Focus only");
    assert!(row(Field::Aura, &base, true).ends_with("Full"));
}

#[test]
fn values_name_the_theme_locale_and_volume() {
    let base = Settings::default();
    assert_eq!(value(Field::Language, &base), "English (US)");
    assert_eq!(value(Field::Theme, &base), "Dark");
    assert_eq!(value(Field::Volume, &base), "70%");
    assert_eq!(value(Field::Cues, &base), "On");
}

#[test]
fn unavailable_sound_rows_say_so_in_their_text() {
    let base = Settings::default();
    assert!(row(Field::Music, &base, false).ends_with("(unavailable)"));
    assert!(!row(Field::Music, &base, true).contains("unavailable"));
    assert!(!row(Field::Theme, &base, false).contains("unavailable"));
}

#[test]
fn every_row_changes_by_keyboard_and_reaches_storage() {
    for tab in [Tab::AudioVisual, Tab::Gameplay] {
        for (index, field) in tab
            .fields()
            .iter()
            .enumerate()
            .filter(|(_, field)| !is_console_font_row(**field))
        {
            let mut h = open_settings();
            switch_to_tab(&mut h, tab);
            move_down(&mut h, index);
            let before = h.app.settings().clone();
            h.press(KeyCode::Enter);
            let expected = changed(*field, &before, Step::Next);
            assert_eq!(h.app.settings(), &expected, "{tab:?} {field:?}");
            let saved = h.storage.borrow().settings.clone();
            if field.is_interface() {
                assert_eq!(saved, None, "{field:?} is saved only when it is kept");
            } else {
                assert_eq!(saved, Some(expected), "{tab:?} {field:?}");
            }
        }
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
    focus_in_tab(&mut h, Tab::Popular, Field::Volume);
    for _ in 0..10 {
        h.press(KeyCode::Left);
    }
    assert_eq!(h.app.settings().volume, 0);
    for _ in 0..12 {
        h.press(KeyCode::Right);
    }
    assert_eq!(h.app.settings().volume, 100);
    assert!(bare(&line_of(&mut h, "Volume")).ends_with("100%"));
}

#[test]
fn a_theme_change_is_shown_at_once() {
    let mut h = open_settings();
    focus_in_tab(&mut h, Tab::Popular, Field::Theme);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().theme, ThemeId::Light);
    assert!(bare(&line_of(&mut h, "Theme")).ends_with("Light"));
}

#[test]
fn a_language_change_switches_the_locale_at_once() {
    let mut h = open_settings();
    h.press(KeyCode::Right);
    assert_eq!(locale(), Locale::PlPl);
    let expected = Locale::PlPl.own_name();
    assert!(bare(&line_of(&mut h, &t("settings.row.language"))).ends_with(expected));
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
    assert!(h.text().contains("► Language"));
    let mut ascii = Harness::with(MemoryStorage::new(), PLAIN, 120, 40);
    move_down(&mut ascii, settings_menu_index());
    ascii.press(KeyCode::Enter);
    let text = ascii.text();
    assert!(text.contains("> Language"));
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
    move_down(&mut h, settings_menu_index());
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.settings().locale, Locale::PlPl);
    assert_eq!(h.app.modal(), Some(&Modal::alert("common.error.storage")));
    assert_eq!(h.app.screen(), Screen::Settings);
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    move_down(&mut h, 1);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.settings().theme, ThemeId::Light);
    assert!(h.app.modal().is_none());
}

#[test]
fn a_settings_write_failure_alerts_once_and_again_after_a_successful_write() {
    let mut h = open_settings();
    h.storage.borrow_mut().failing = true;
    h.press(KeyCode::Enter);
    assert_eq!(h.app.modal(), Some(&Modal::alert("common.error.storage")));
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    h.storage.borrow_mut().failing = false;
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    h.storage.borrow_mut().failing = true;
    h.press(KeyCode::Enter);
    assert_eq!(h.app.modal(), Some(&Modal::alert("common.error.storage")));
}

#[test]
fn the_row_text_uses_the_line_format() {
    let base = Settings::default();
    assert!(row(Field::Volume, &base, true).starts_with("Volume "));
    assert!(row(Field::Volume, &base, true).ends_with(" 70%"));
    assert_eq!(t("settings.row.line"), "{label} {value}");
}

#[test]
fn the_focused_row_is_explained_in_the_description_line() {
    let mut h = open_settings();
    assert!(h.text().contains(&*t("settings.desc.language")));
    focus_in_tab(&mut h, Tab::Popular, Field::Theme);
    assert!(h.text().contains(&*t("settings.desc.theme")));
    assert!(!h.text().contains(&*t("settings.desc.language")));
}

#[test]
fn a_click_on_a_row_reaches_the_settings_through_the_app() {
    let mut h = open_settings();
    let found = h.find("Theme");
    assert!(found.is_some());
    let (column, row) = found.unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.settings().theme, ThemeId::Light);
    assert_eq!(h.storage.borrow().settings, None);
}
