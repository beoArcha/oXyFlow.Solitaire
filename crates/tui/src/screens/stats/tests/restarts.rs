//! Restarts in the statistics (P4-05): wins without a restart beside all wins for games without
//! undo, and the restart count in the history. Everything fits 100 columns in both locales.

use oxysol_core::{PyramidRules, Variant};

use super::support::{many_games, open_stats, row};
use super::{KLONDIKE, record};
use crate::glyphs::Glyphs;
use crate::i18n::Locale;
use crate::screens::stats::format::{history_header, history_row, summary_lines};
use crate::screens::stats::summary::summarize;
use crate::settings::{GlyphMode, Settings};
use crate::storage::Outcome::{Abandoned, Won};
use crate::storage::memory::HistoryRow;
use crate::storage::{GameCounters, GameRecord};

fn played_without_undo(id: i64, restarts: u32) -> GameRecord {
    GameRecord {
        undo_allowed: false,
        counters: GameCounters {
            restarts,
            ..GameCounters::default()
        },
        ..record(id, KLONDIKE, Some(Won), Some(60_000), Some(50))
    }
}

#[test]
fn wins_without_a_restart_are_counted_apart_for_games_without_undo() {
    let history = [
        played_without_undo(1, 0),
        played_without_undo(2, 2),
        played_without_undo(3, 0),
        GameRecord {
            outcome: Some(Abandoned),
            ..played_without_undo(4, 0)
        },
        record(5, KLONDIKE, Some(Won), Some(1), Some(1)),
    ];
    let summaries = summarize(&history);
    let (undo, no_undo) = (&summaries[0], &summaries[1]);
    assert!(undo.undo_allowed && !no_undo.undo_allowed);
    assert_eq!(undo.clean_wins, None);
    assert_eq!((no_undo.played, no_undo.won), (4, 3));
    assert_eq!(no_undo.clean_wins, Some(2));
}

#[test]
fn the_clean_wins_column_exists_only_when_a_game_without_undo_was_finished() {
    let undo_only = summarize(&[record(1, KLONDIKE, Some(Won), Some(1), Some(1))]);
    let lines = summary_lines(&undo_only, false, true);
    assert!(!lines[0].contains("No restart"), "{lines:?}");
    let mixed = summarize(&[
        record(1, KLONDIKE, Some(Won), Some(1), Some(1)),
        played_without_undo(2, 1),
        played_without_undo(3, 0),
    ]);
    let lines = summary_lines(&mixed, true, true);
    assert!(lines[0].contains("Won No restart"), "{lines:?}");
    let words =
        |line: &str| -> Vec<String> { line.split_whitespace().map(str::to_owned).collect() };
    assert_eq!(words(&lines[1])[3..8], ["(undo)", "1", "1", "-", "100%"]);
    assert_eq!(
        words(&lines[2])[3..9],
        ["(no", "undo)", "2", "2", "1", "100%"]
    );
}

#[test]
fn the_history_lists_the_restart_count_of_a_game_without_undo() {
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    assert!(
        history_header(&glyphs, false)
            .trim_end()
            .ends_with("Restarts")
    );
    let restarted = history_row(&played_without_undo(2, 4), &glyphs, false);
    assert!(restarted.trim_end().ends_with(" 4"), "{restarted:?}");
    let with_undo = history_row(
        &record(2, KLONDIKE, Some(Won), Some(1), Some(1)),
        &glyphs,
        false,
    );
    assert!(with_undo.trim_end().ends_with(" -"), "{with_undo:?}");
    assert_eq!(
        restarted.chars().count(),
        history_header(&glyphs, false).chars().count()
    );
    assert_eq!(
        with_undo.chars().count(),
        history_header(&glyphs, false).chars().count()
    );
}

#[test]
fn the_summary_and_the_history_show_restarts_at_100_by_30_in_both_locales() {
    for (locale, date, history, clean, restarts) in [
        (Locale::EnUs, "Date", "Restarts", "No restart", "(no undo)"),
        (Locale::PlPl, "Data", "Od nowa", "Czyste", "(bez cofania)"),
    ] {
        let mut storage = many_games(3);
        for (index, variant) in Variant::ALL.into_iter().enumerate() {
            let id = 10 + i64::try_from(index).unwrap_or(0);
            storage.history.push(HistoryRow {
                undo_allowed: false,
                counters: GameCounters {
                    restarts: 3,
                    ..GameCounters::default()
                },
                ..row(id, variant, Won, 61_000)
            });
        }
        storage.history.push(HistoryRow {
            undo_allowed: false,
            counters: GameCounters {
                restarts: 12,
                ..GameCounters::default()
            },
            seed: u64::MAX,
            ..row(
                30,
                Variant::Pyramid(PyramidRules::Classic),
                Abandoned,
                3_725_000,
            )
        });
        storage.settings = Some(Settings {
            locale,
            ..Settings::default()
        });
        let mut h = open_stats(storage, 100, 30);
        let lines = h.lines();
        let text = lines.join("\n");
        assert!(text.contains(history), "{text}");
        assert!(text.contains(clean), "{text}");
        assert!(text.contains(restarts), "{text}");
        let widest = lines
            .iter()
            .map(|line| line.trim_end().chars().count())
            .max();
        assert!(widest.is_some_and(|width| width <= 100), "{text}");
        let top = lines
            .iter()
            .find(|line| line.contains(date))
            .cloned()
            .unwrap_or_default();
        assert!(top.trim_end().ends_with(history), "{top:?}");
    }
}

#[test]
fn the_longest_streak_column_is_shown_from_110_columns() {
    for (locale, longest) in [(Locale::EnUs, "Best streak"), (Locale::PlPl, "Rekord")] {
        let storage = || {
            let mut storage = many_games(3);
            storage.settings = Some(Settings {
                locale,
                ..Settings::default()
            });
            storage
        };
        let mut narrow = open_stats(storage(), 100, 30);
        assert!(!narrow.text().contains(longest));
        let mut wide = open_stats(storage(), 110, 30);
        assert!(wide.text().contains(longest), "{}", wide.text());
    }
}

#[test]
fn a_row_with_restarts_stays_as_wide_as_the_header_in_both_locales() {
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    for locale in Locale::offered() {
        crate::i18n::set_locale(locale);
        let worst = GameRecord {
            seed: u64::MAX,
            elapsed_ms: Some(35_999_000),
            moves: Some(99_999),
            variant: Variant::Pyramid(PyramidRules::Classic),
            counters: GameCounters {
                restarts: 999,
                ..GameCounters::default()
            },
            ..played_without_undo(1, 0)
        };
        for short in [false, true] {
            assert_eq!(
                history_row(&worst, &glyphs, short).chars().count(),
                history_header(&glyphs, short).chars().count()
            );
            assert!(
                history_header(&glyphs, short).chars().count() <= 100,
                "{}",
                history_header(&glyphs, short)
            );
        }
    }
    crate::i18n::set_locale(Locale::EnUs);
}
