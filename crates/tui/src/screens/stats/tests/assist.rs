//! The assist counters in the statistics (P4-06): the assisted count in the summary, and the assist
//! line of the selected game. Both fit their terminal width in every offered locale.

use oxysol_core::Variant;

use super::support::{many_games, open_stats, row};
use super::{KLONDIKE, record};
use crate::i18n::{Locale, set_locale};
use crate::screens::stats::assist::assist_line;
use crate::screens::stats::format::summary_lines;
use crate::screens::stats::summary::summarize;
use crate::settings::Settings;
use crate::storage::counters::{ASSISTED_ABOVE_PERCENT, ASSISTED_MIN_MOVES};
use crate::storage::memory::HistoryRow;
use crate::storage::{AssistCounters, GameCounters, GameRecord, Outcome};

fn with_assist(id: i64, assist: AssistCounters) -> GameRecord {
    GameRecord {
        counters: GameCounters {
            assist,
            ..GameCounters::default()
        },
        ..record(id, KLONDIKE, Some(Outcome::Won), Some(60_000), Some(50))
    }
}

fn share_of(quick: u32, total: u32) -> AssistCounters {
    AssistCounters {
        quick,
        manual: total - quick,
        ..AssistCounters::default()
    }
}

fn worst() -> AssistCounters {
    AssistCounters {
        manual: 1,
        quick: 99_999,
        finish: 99_999,
        failed: 99_999,
        longest_streak: 99_999,
    }
}

#[test]
fn the_summary_counts_the_assisted_games_at_the_threshold() {
    let total = 10 * ASSISTED_MIN_MOVES;
    let at = total * ASSISTED_ABOVE_PERCENT / 100;
    let history = [
        with_assist(1, share_of(at, total)),
        with_assist(2, share_of(at + 1, total)),
        with_assist(3, share_of(total, total)),
        with_assist(4, AssistCounters::default()),
    ];
    let summaries = summarize(&history);
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].assisted, 2);
    assert_eq!(summaries[0].played, 4);
}

#[test]
fn the_assisted_column_is_shown_from_110_columns_in_every_offered_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let summaries = summarize(&[with_assist(1, worst())]);
        let wide = summary_lines(&summaries, true, true);
        let narrow = summary_lines(&summaries, true, false);
        let width = |lines: &[String]| lines.iter().map(|line| line.chars().count()).max();
        assert!(width(&wide) > width(&narrow), "{locale:?}");
        assert!(width(&wide).is_some_and(|w| w <= 110), "{wide:?}");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_line_names_the_share_the_failures_the_run_and_the_finish() {
    set_locale(Locale::EnUs);
    let line = assist_line(&AssistCounters {
        manual: 4,
        quick: 6,
        finish: 12,
        failed: 3,
        longest_streak: 5,
    });
    assert_eq!(
        line.as_deref(),
        Some("Automatic 60% (6 of 10) · failed 3 · run 5 · finish 12")
    );
    assert_eq!(assist_line(&AssistCounters::default()), None);
}

#[test]
fn the_assisted_tag_appears_only_above_the_threshold() {
    set_locale(Locale::EnUs);
    let total = 10 * ASSISTED_MIN_MOVES;
    let at = total * ASSISTED_ABOVE_PERCENT / 100;
    let tag = "assisted game";
    assert!(!assist_line(&share_of(at, total)).is_some_and(|line| line.contains(tag)));
    assert!(assist_line(&share_of(at + 1, total)).is_some_and(|line| line.contains(tag)));
}

#[test]
fn the_line_fits_100_columns_in_every_offered_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let line = assist_line(&worst());
        assert!(
            line.as_ref().is_some_and(|l| l.chars().count() <= 100),
            "{locale:?}: {line:?}"
        );
    }
    set_locale(Locale::EnUs);
}

fn assisted_row(id: i64) -> HistoryRow {
    HistoryRow {
        counters: GameCounters {
            assist: worst(),
            ..GameCounters::default()
        },
        ..row(id, Variant::FreeCell, Outcome::Won, 61_000)
    }
}

#[test]
fn the_selected_game_shows_its_line_and_a_game_without_counters_shows_the_check_keys() {
    let mut storage = many_games(2);
    storage.history.push(assisted_row(9));
    let mut h = open_stats(storage, 100, 30);
    let text = h.text();
    assert!(text.contains("Automatic 100% (99999 of 100000)"), "{text}");
    assert!(text.contains("assisted game"), "{text}");
    assert!(!text.contains("w: check this game"), "{text}");
    h.press(ratatui::crossterm::event::KeyCode::Down);
    let text = h.text();
    assert!(text.contains("w: check this game"), "{text}");
    assert!(!text.contains("Automatic"), "{text}");
}

#[test]
fn the_screen_shows_the_line_within_100_columns_in_every_offered_locale() {
    for locale in Locale::offered() {
        let mut storage = many_games(1);
        storage.history.insert(0, assisted_row(9));
        storage.settings = Some(Settings {
            locale,
            ..Settings::default()
        });
        let mut h = open_stats(storage, 100, 30);
        let widest = h
            .lines()
            .iter()
            .map(|line| line.trim_end().chars().count())
            .max();
        assert!(widest.is_some_and(|width| width <= 100), "{locale:?}");
    }
}
