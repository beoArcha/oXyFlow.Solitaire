//! The assist counters in the statistics (P4-06): the assisted count in the summary, and the assist
//! line of the selected game. Both fit their terminal width in every offered locale.

use oxysol_core::Variant;

use super::support::{many_games, open_stats, row};
use super::{KLONDIKE, record};
use crate::i18n::{Locale, set_locale, t, tf};
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
    for locale in Locale::offered() {
        set_locale(locale);
        let line = assist_line(&AssistCounters {
            manual: 4,
            quick: 6,
            finish: 12,
            failed: 3,
            longest_streak: 5,
        });
        let expected = [
            tf(
                "stats.assist.share",
                &[("share", "60"), ("auto", "6"), ("total", "10")],
            ),
            tf("stats.assist.failed", &[("failed", "3")]),
            tf("stats.assist.run", &[("run", "5")]),
            tf("stats.assist.finish", &[("finish", "12")]),
        ]
        .join(" · ");
        assert_eq!(line.as_deref(), Some(expected.as_str()), "{locale:?}");
        assert_eq!(assist_line(&AssistCounters::default()), None);
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_assisted_tag_appears_only_above_the_threshold() {
    for locale in Locale::offered() {
        set_locale(locale);
        let total = 10 * ASSISTED_MIN_MOVES;
        let at = total * ASSISTED_ABOVE_PERCENT / 100;
        let tag = t("stats.assist.tag");
        assert!(
            !assist_line(&share_of(at, total)).is_some_and(|line| line.contains(tag.as_ref())),
            "{locale:?}"
        );
        assert!(
            assist_line(&share_of(at + 1, total)).is_some_and(|line| line.contains(tag.as_ref())),
            "{locale:?}"
        );
    }
    set_locale(Locale::EnUs);
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
    set_locale(Locale::EnUs);
    let expected_share = tf(
        "stats.assist.share",
        &[("share", "100"), ("auto", "99999"), ("total", "100000")],
    );
    let expected_tag = t("stats.assist.tag").into_owned();
    let expected_hint = t("stats.check.hint").into_owned();
    let mut storage = many_games(2);
    storage.history.push(assisted_row(9));
    let mut h = open_stats(storage, 100, 60);
    let text = h.text();
    assert!(text.contains(&expected_share), "{text}");
    assert!(text.contains(&expected_tag), "{text}");
    assert!(!text.contains(&expected_hint), "{text}");
    h.press(ratatui::crossterm::event::KeyCode::Down);
    let text = h.text();
    assert!(text.contains(&expected_hint), "{text}");
    assert!(!text.contains(&expected_share), "{text}");
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
