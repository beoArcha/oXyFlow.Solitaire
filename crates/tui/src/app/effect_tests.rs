use super::*;
use crate::audio::AudioSettings;
use crate::i18n::Locale;
use crate::input::Input;
use crate::settings::{ColorMode, GlyphMode};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use crate::theme::ThemeId;
use oxysol_core::{DrawCount, Session, Variant};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};

const FIRST_VARIANT: Variant = Variant::Klondike(DrawCount::One);

fn start_first_game(h: &mut Harness) {
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
}

fn saved_game(variant: Variant, seed: u64) -> SavedGame {
    SavedGame {
        id: crate::storage::GameId(7),
        record: Session::new(variant, seed).record(),
        elapsed_ms: 90_000,
        undo_allowed: true,
    }
}

#[test]
fn starting_a_game_uses_the_injected_seed_and_records_it() {
    let mut h = Harness::new();
    start_first_game(&mut h);
    assert_eq!(h.app.screen(), Screen::Board);
    let session = h.app.board().session();
    assert!(session.is_some_and(|s| s.variant() == FIRST_VARIANT && s.seed() == 42));
    let history = h.storage.borrow().history.clone();
    assert_eq!(history.len(), 1);
    assert_eq!((history[0].variant, history[0].seed), (FIRST_VARIANT, 42));
    assert!(h.text().contains("Klondike, Draw 1 | seed 42 | Time 0:00"));
}

#[test]
fn every_variant_can_be_started_from_the_list() {
    for (index, variant) in Variant::ALL.into_iter().enumerate() {
        let mut h = Harness::new();
        h.press(KeyCode::Enter);
        for _ in 0..index {
            h.press(KeyCode::Down);
        }
        h.press(KeyCode::Enter);
        let session = h.app.board().session();
        assert!(
            session.is_some_and(|s| s.variant() == variant),
            "{variant:?}"
        );
    }
}

#[test]
fn a_game_that_cannot_be_recorded_still_starts_with_an_alert() {
    let mut h = Harness::new();
    h.storage.borrow_mut().failing = true;
    start_first_game(&mut h);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(matches!(h.app.modal(), Some(Modal::Alert { .. })));
    assert!(h.app.board().game_id().is_none());
    assert!(h.text().contains("Saving to the database failed"));
}

#[test]
fn the_clock_runs_on_the_board_and_stops_in_modals() {
    let mut h = Harness::new();
    start_first_game(&mut h);
    h.advance(65);
    assert!(h.text().contains("seed 42 | Time 1:05"));
    h.type_char('?');
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    h.advance(30);
    h.press(KeyCode::Esc);
    h.advance(5);
    assert!(h.text().contains("seed 42 | Time 1:10"));
}

#[test]
fn leaving_the_board_stops_the_clock() {
    let mut h = Harness::new();
    start_first_game(&mut h);
    h.advance(10);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(!h.app.board().clock_running());
}

#[test]
fn help_modal_takes_input_and_closes_on_escape() {
    let mut h = Harness::new();
    h.type_char('?');
    assert!(h.text().contains("Help"));
    h.type_char('q');
    assert!(!h.app.should_quit());
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
}

#[test]
fn a_confirm_modal_applies_its_effect_only_after_yes() {
    let mut h = Harness::new();
    h.app.apply(Effect::Open(Modal::confirm(
        "common.dialog.confirm_title",
        Effect::Quit,
    )));
    assert!(h.text().contains("No"));
    h.press(KeyCode::Enter);
    assert!(!h.app.should_quit());
    h.app.apply(Effect::Open(Modal::confirm(
        "common.dialog.confirm_title",
        Effect::Quit,
    )));
    h.press(KeyCode::Left);
    h.press(KeyCode::Enter);
    assert!(h.app.should_quit());
}

#[test]
fn applying_settings_saves_them_and_reconfigures_theme_language_and_audio() {
    let mut h = Harness::new();
    let settings = Settings {
        locale: Locale::PlPl,
        theme: ThemeId::HighContrast,
        glyph_mode: GlyphMode::Ascii,
        color_mode: ColorMode::Ansi16,
        volume: 30,
        ..Settings::default()
    };
    h.app.apply(Effect::Apply(settings.clone()));
    assert_eq!(h.app.settings(), &settings);
    assert_eq!(h.storage.borrow().settings.as_ref(), Some(&settings));
    assert_eq!(h.app.theme.id(), ThemeId::HighContrast);
    assert_eq!(
        h.app.glyphs,
        Glyphs::styled(GlyphMode::Ascii, "international", "lattice")
    );
    assert_eq!(
        h.app.theme.color(crate::theme::Token::Text),
        ratatui::style::Color::White
    );
    assert_eq!(i18n::locale(), Locale::PlPl);
    let configured = h.audio.borrow().configured.clone();
    let last: Option<&AudioSettings> = configured.last();
    assert!(last.is_some_and(|a| a.volume == 30));
}

#[test]
fn startup_configures_the_audio_once() {
    let h = Harness::new();
    assert_eq!(h.audio.borrow().configured.len(), 1);
}

#[test]
fn stored_settings_are_loaded_at_startup() {
    let stored = Settings {
        theme: ThemeId::Light,
        ..Settings::default()
    };
    let storage = MemoryStorage {
        settings: Some(stored.clone()),
        ..MemoryStorage::new()
    };
    let h = Harness::with(storage, TRUE_COLOR, 120, 40);
    assert_eq!(h.app.settings(), &stored);
}

#[test]
fn a_failing_save_keeps_the_setting_and_alerts() {
    let mut h = Harness::new();
    h.storage.borrow_mut().failing = true;
    let settings = Settings {
        theme: ThemeId::Light,
        ..Settings::default()
    };
    h.app.apply(Effect::Apply(settings.clone()));
    assert_eq!(h.app.settings(), &settings);
    assert!(matches!(h.app.modal(), Some(Modal::Alert { .. })));
}

#[test]
fn unreadable_storage_at_startup_opens_an_alert_and_uses_defaults() {
    let storage = MemoryStorage {
        failing: true,
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    assert_eq!(h.app.settings(), &Settings::for_caps(&TRUE_COLOR));
    assert!(h.text().contains("Stored settings could not be read"));
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
}

#[test]
fn a_saved_game_can_be_resumed() {
    let storage = MemoryStorage {
        game: Some(saved_game(Variant::FreeCell, 99)),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Resume);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(h.app.board().session().is_some_and(|s| s.seed() == 99));
    assert_eq!(h.app.board().game_id(), Some(crate::storage::GameId(7)));
    h.advance(1);
    assert!(h.text().contains("FreeCell | seed 99 | Time 1:31"));
}

#[test]
fn an_unrestorable_saved_game_alerts_instead_of_starting() {
    let mut saved = saved_game(Variant::FreeCell, 99);
    saved.record.cursor = 5;
    let storage = MemoryStorage {
        game: Some(saved),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Resume);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(matches!(h.app.modal(), Some(Modal::Alert { .. })));
}

#[test]
fn a_typed_seed_bypasses_the_seed_source() {
    let mut h = Harness::new();
    h.app.apply(Effect::Start {
        variant: Variant::FreeCell,
        seed: Some(7),
        undo_allowed: true,
    });
    assert!(h.app.board().session().is_some_and(|s| s.seed() == 7));
    h.app.apply(Effect::Start {
        variant: Variant::FreeCell,
        seed: None,
        undo_allowed: true,
    });
    assert!(h.app.board().session().is_some_and(|s| s.seed() == 42));
}

fn hover_at(h: &mut Harness, column: u16, row: u16) {
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

#[test]
fn a_window_hides_the_language_tooltip_until_the_next_hover_or_focus() {
    let mut h = Harness::new();
    let name = h.app.settings().locale.own_name();
    let Some((column, row)) = h.find("▀") else {
        panic!("the language selector is not drawn");
    };
    hover_at(&mut h, column + 2, row);
    assert!(h.text().contains(name));
    h.type_char('?');
    assert!(h.app.modal().is_some());
    assert!(!h.text().contains(name));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert!(!h.text().contains(name));
    hover_at(&mut h, column + 2, row);
    assert!(h.text().contains(name));
}

#[test]
fn a_window_hides_the_language_tooltip_shown_by_keyboard_focus_too() {
    let mut h = Harness::new();
    let name = h.app.settings().locale.own_name();
    h.press(KeyCode::BackTab);
    assert!(h.text().contains(name));
    h.type_char('?');
    assert!(!h.text().contains(name));
    h.press(KeyCode::Esc);
    assert!(!h.text().contains(name));
    h.press(KeyCode::BackTab);
    assert!(h.text().contains(name));
}
