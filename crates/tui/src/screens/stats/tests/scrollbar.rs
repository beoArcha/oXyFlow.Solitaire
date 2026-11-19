//! Tests for the history list scroll bar, winnable marks, and legend (O-33).

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::support::{KLONDIKE, many_games, open_stats, row, wheel};
use crate::glyphs::Glyphs;
use crate::i18n::Locale;
use crate::input::Input;
use crate::settings::{GlyphMode, Settings};
use crate::solver::Proof;
use crate::storage::Outcome::Won;
use crate::storage::memory::{HistoryRow, MemoryStorage};

fn storage_with_proofs(mode: GlyphMode, locale: Locale) -> MemoryStorage {
    let proofs = [
        Some(Proof::Winnable),
        Some(Proof::Unwinnable),
        Some(Proof::Unknown),
        None,
    ];
    let history = proofs
        .into_iter()
        .enumerate()
        .map(|(index, proof)| {
            let id = i64::try_from(index).unwrap_or(0) + 1;
            HistoryRow {
                proof,
                ..row(id, KLONDIKE, Won, 60_000)
            }
        })
        .collect();
    MemoryStorage {
        history,
        settings: Some(Settings {
            glyph_mode: mode,
            locale,
            ..Settings::default()
        }),
        ..MemoryStorage::new()
    }
}

#[test]
fn the_history_scrollbar_draws_thumb_and_groove_in_both_glyph_modes() {
    for (mode, thumb, groove) in [(GlyphMode::Unicode, "█", "░"), (GlyphMode::Ascii, "#", ":")]
    {
        let mut storage = many_games(60);
        storage.settings = Some(Settings {
            glyph_mode: mode,
            ..Settings::default()
        });
        let mut h = open_stats(storage, 100, 30);
        let text = h.text();
        assert!(text.contains(thumb), "missing thumb {thumb} for {mode:?}");
        assert!(
            text.contains(groove),
            "missing groove {groove} for {mode:?}"
        );
    }
}

#[test]
fn mouse_track_click_pages_and_drag_scrolls_the_history() {
    let mut storage = many_games(60);
    storage.settings = Some(Settings::default());
    let mut h = open_stats(storage, 100, 30);
    assert!(h.text().contains("Game 1 of 60"));

    let lines = h.lines();
    let track_x = lines[7]
        .chars()
        .position(|ch| ch == '█' || ch == '░')
        .map(|x| u16::try_from(x).unwrap_or(99))
        .unwrap_or(99);

    wheel(&mut h, MouseEventKind::ScrollDown);
    assert!(
        h.text().contains("Game 2 of 60"),
        "wheel did not scroll down"
    );

    assert!(
        h.text().contains(" 660 "),
        "first row missing before page down"
    );
    h.click(track_x, 20);
    assert!(!h.text().contains(" 660 "), "track click did not page down");

    h.click(track_x, 7);
    assert!(
        h.text().contains(" 660 "),
        "track click did not page back up"
    );

    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: track_x,
        row: 7,
        modifiers: KeyModifiers::NONE,
    }));
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: track_x,
        row: 20,
        modifiers: KeyModifiers::NONE,
    }));
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: track_x,
        row: 20,
        modifiers: KeyModifiers::NONE,
    }));
    assert!(!h.text().contains(" 660 "), "drag did not scroll down");
}

#[test]
fn winnable_marks_differ_for_each_proof_in_both_glyph_modes() {
    let glyphs_u = Glyphs::new(GlyphMode::Unicode);
    assert_eq!(glyphs_u.proof_mark(Some(Proof::Winnable)), "✓");
    assert_eq!(glyphs_u.proof_mark(Some(Proof::Unwinnable)), "✗");
    assert_eq!(glyphs_u.proof_mark(Some(Proof::Unknown)), "?");
    assert_eq!(glyphs_u.proof_mark(None), "·");

    let glyphs_a = Glyphs::new(GlyphMode::Ascii);
    assert_eq!(glyphs_a.proof_mark(Some(Proof::Winnable)), "+");
    assert_eq!(glyphs_a.proof_mark(Some(Proof::Unwinnable)), "x");
    assert_eq!(glyphs_a.proof_mark(Some(Proof::Unknown)), "?");
    assert_eq!(glyphs_a.proof_mark(None), ".");

    let mut h_u = open_stats(
        storage_with_proofs(GlyphMode::Unicode, Locale::EnUs),
        100,
        30,
    );
    let text_u = h_u.text();
    assert!(text_u.contains('✓'));
    assert!(text_u.contains('✗'));
    assert!(text_u.contains('·'));

    let mut h_a = open_stats(storage_with_proofs(GlyphMode::Ascii, Locale::EnUs), 100, 30);
    let text_a = h_a.text();
    assert!(text_a.contains('+'));
    assert!(text_a.contains('.'));
}

#[test]
fn the_legend_explains_all_signs_in_both_locales_and_glyph_modes() {
    let cases = [
        (
            GlyphMode::Unicode,
            Locale::EnUs,
            "✓ winnable",
            "✗ not winnable",
            "? unknown",
            "· not checked",
        ),
        (
            GlyphMode::Ascii,
            Locale::EnUs,
            "+ winnable",
            "x not winnable",
            "? unknown",
            ". not checked",
        ),
        (
            GlyphMode::Unicode,
            Locale::PlPl,
            "✓ do wygrania",
            "✗ nie do wygrania",
            "? nieznane",
            "· niesprawdzone",
        ),
        (
            GlyphMode::Ascii,
            Locale::PlPl,
            "+ do wygrania",
            "x nie do wygrania",
            "? nieznane",
            ". niesprawdzone",
        ),
    ];
    for (mode, locale, win, unwin, unk, unchk) in cases {
        let storage = storage_with_proofs(mode, locale);
        let mut h = open_stats(storage, 100, 30);
        let text = h.text();
        assert!(text.contains(win), "missing {win} in {locale:?} {mode:?}");
        assert!(
            text.contains(unwin),
            "missing {unwin} in {locale:?} {mode:?}"
        );
        assert!(text.contains(unk), "missing {unk} in {locale:?} {mode:?}");
        assert!(
            text.contains(unchk),
            "missing {unchk} in {locale:?} {mode:?}"
        );
    }
}
