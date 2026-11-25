use std::time::Duration;

use oxysol_core::{DrawCount, Session, Variant};
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

use super::resume::label;
use crate::app::Screen;
use crate::caps::ColorDepth;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::SavedSummary;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::storage::{GameId, SavedGame};
use crate::test_support::{Harness, TRUE_COLOR};
use crate::theme::{Theme, Token};

fn summary(variant: Variant) -> SavedSummary {
    SavedSummary {
        variant,
        elapsed: Duration::from_secs(754),
        moves: 42,
    }
}

fn harness_with_save(locale: Locale) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            ..Settings::for_caps(&TRUE_COLOR)
        }),
        game: Some(SavedGame {
            id: GameId(3),
            record: Session::new(Variant::FreeCell, 5).record(),
            elapsed_ms: 61_000,
            undo_allowed: true,
        }),
        ..MemoryStorage::new()
    };
    Harness::with(storage, TRUE_COLOR, 100, 30)
}

fn row_of(lines: &[String], needle: &str) -> usize {
    lines
        .iter()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no row shows {needle:?}"))
}

const CORNER_ROWS: usize = 3;

/// The first row of the list that carries the focus marker; the language selector sits above it.
fn focus_row(lines: &[String]) -> Option<&String> {
    lines.iter().skip(CORNER_ROWS).find(|line| line.contains('►'))
}

fn foreground_at(h: &mut Harness, column: u16, row: u16) -> Color {
    h.lines();
    h.terminal.backend().buffer()[(column, row)].fg
}

/// The first cell after the word `Resume` on its row, where the muted details begin.
fn resume_detail_column(h: &mut Harness) -> (u16, u16) {
    let (column, row) = h.find("Resume:").expect("the Resume row is drawn");
    let word = u16::try_from("Resume".len()).unwrap_or(0);
    (column + word, row)
}

#[test]
fn resume_names_the_game_its_time_and_its_moves_at_100_columns() {
    set_locale(Locale::EnUs);
    let klondike = summary(Variant::Klondike(DrawCount::Three));
    assert_eq!(
        label(klondike, 100),
        "Resume: Klondike, Draw 3, 12:34, Moves 42"
    );
    set_locale(Locale::PlPl);
    assert_eq!(
        label(klondike, 100),
        "Wznów: Klondike, 3 karty, 12:34, Ruchy 42"
    );
    set_locale(Locale::EnUs);
}

#[test]
fn a_narrow_row_drops_the_moves_then_the_time() {
    set_locale(Locale::EnUs);
    let klondike = summary(Variant::Klondike(DrawCount::Three));
    assert_eq!(label(klondike, 40), "Resume: Klondike, Draw 3, 12:34");
    assert_eq!(label(klondike, 28), "Resume: Klondike, Draw 3");
    assert_eq!(label(klondike, 4), "Resume: Klondike, Draw 3");
}

#[test]
fn the_saved_game_is_named_in_the_menu() {
    set_locale(Locale::EnUs);
    let mut h = harness_with_save(Locale::EnUs);
    assert!(h.text().contains("Resume: FreeCell, 1:01, Moves 0"));
}

#[test]
fn an_empty_row_separates_resume_from_the_other_entries() {
    set_locale(Locale::EnUs);
    let mut h = harness_with_save(Locale::EnUs);
    let lines = h.lines();
    let resume = row_of(&lines, "Resume: FreeCell");
    assert!(lines[resume + 1].trim().is_empty());
    assert!(lines[resume + 2].contains("New game"));
}

#[test]
fn the_game_details_are_muted_while_the_row_is_not_selected() {
    set_locale(Locale::EnUs);
    let mut h = harness_with_save(Locale::EnUs);
    h.press(KeyCode::Down);
    let theme = Theme::new(h.app.settings().theme, ColorDepth::TrueColor);
    let (column, row) = resume_detail_column(&mut h);
    let muted = theme.style(Token::TextMuted, Token::Background).fg;
    assert_eq!(Some(foreground_at(&mut h, column, row)), muted);
    assert_ne!(muted, theme.base().fg);
}

#[test]
fn the_selected_resume_row_uses_the_focus_style_throughout() {
    set_locale(Locale::EnUs);
    let mut h = harness_with_save(Locale::EnUs);
    let theme = Theme::new(h.app.settings().theme, ColorDepth::TrueColor);
    let (column, row) = resume_detail_column(&mut h);
    assert_eq!(Some(foreground_at(&mut h, column, row)), theme.focus().fg);
}

#[test]
fn a_click_on_the_empty_row_does_nothing() {
    set_locale(Locale::EnUs);
    let mut h = harness_with_save(Locale::EnUs);
    let lines = h.lines();
    let gap = row_of(&lines, "Resume: FreeCell") + 1;
    h.click(50, u16::try_from(gap).unwrap_or(0));
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn a_click_on_the_item_after_the_gap_activates_that_item() {
    set_locale(Locale::EnUs);
    let mut h = harness_with_save(Locale::EnUs);
    let (column, row) = h.find("New game").expect("New game is drawn");
    h.click(column, row);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn keyboard_focus_skips_the_empty_row() {
    set_locale(Locale::EnUs);
    let mut h = harness_with_save(Locale::EnUs);
    h.press(KeyCode::Down);
    let lines = h.lines();
    let marked = focus_row(&lines).expect("a row carries the focus marker");
    assert!(marked.contains("New game"));
    h.press(KeyCode::Up);
    let lines = h.lines();
    assert!(focus_row(&lines).is_some_and(|line| line.contains("Resume")));
}

#[test]
fn the_menu_fits_at_100_by_30_in_every_offered_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = harness_with_save(locale);
        let lines = h.lines();
        let resume_text = label(
            SavedSummary {
                variant: Variant::FreeCell,
                elapsed: Duration::from_secs(61),
                moves: 0,
            },
            100,
        );
        let resume = row_of(&lines, &resume_text);
        let quit = t("common.menu.quit").into_owned();
        let quit_row = row_of(&lines, &quit);
        assert_eq!(quit_row, resume + 8, "{locale:?}: Quit below the gap");
        assert!(lines[resume + 1].trim().is_empty(), "{locale:?}");
    }
    set_locale(Locale::EnUs);
}
