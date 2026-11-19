//! Text of the statistics: dates, durations and the rows of the two tables.

use std::time::Duration;

use super::summary::Summary;
use crate::clock::format_elapsed;
use crate::i18n::t;
use crate::screens::variant_label_key;
use crate::solver::Proof;
use crate::storage::{GameRecord, Outcome};
use oxysol_core::Variant;

const NONE: &str = "-";
const MILLIS_PER_DAY: i64 = 86_400_000;
const MILLIS_PER_MINUTE: i64 = 60_000;
const COLUMN_GAP: &str = " ";

/// Year, month and day of a day count since 1970-01-01 in the proleptic Gregorian calendar.
fn civil_date(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

/// `YYYY-MM-DD HH:MM` in UTC for Unix milliseconds. The standard library cannot convert to the
/// local time zone, so the history is shown in UTC and its heading says so.
pub fn format_utc(unix_ms: i64) -> String {
    let days = unix_ms.div_euclid(MILLIS_PER_DAY);
    let minutes = unix_ms.rem_euclid(MILLIS_PER_DAY) / MILLIS_PER_MINUTE;
    let (year, month, day) = civil_date(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        minutes / 60,
        minutes % 60
    )
}

/// `m:ss` or `h:mm:ss`; `-` when unknown.
pub fn format_time(elapsed_ms: Option<u64>) -> String {
    elapsed_ms.map_or_else(
        || NONE.to_owned(),
        |ms| format_elapsed(Duration::from_millis(ms)),
    )
}

fn format_count(value: Option<u32>) -> String {
    value.map_or_else(|| NONE.to_owned(), |count| count.to_string())
}

fn outcome_label(outcome: Option<Outcome>) -> String {
    let key = match outcome {
        Some(Outcome::Won) => "stats.outcome.won",
        Some(Outcome::Abandoned) => "stats.outcome.abandoned",
        None => "stats.outcome.open",
    };
    t(key).into_owned()
}

fn winnable_label(proof: Option<Proof>) -> String {
    let key = match proof {
        Some(Proof::Winnable) => "stats.winnable.yes",
        Some(Proof::Unwinnable) => "stats.winnable.no",
        Some(Proof::Unknown) => "stats.winnable.unknown",
        None => "stats.winnable.unchecked",
    };
    t(key).into_owned()
}

fn variant_label(variant: Variant) -> String {
    t(variant_label_key(variant)).into_owned()
}

fn char_width(text: &str) -> usize {
    text.chars().count()
}

fn pad_row(cells: &[String], widths: &[usize], right_from: usize) -> String {
    cells
        .iter()
        .zip(widths)
        .enumerate()
        .map(|(column, (cell, width))| {
            if column >= right_from {
                format!("{cell:>width$}")
            } else {
                format!("{cell:<width$}")
            }
        })
        .collect::<Vec<_>>()
        .join(COLUMN_GAP)
}

/// Pads every column to its widest cell; columns from `right_from` on are right-aligned.
fn table(rows: &[Vec<String>], right_from: usize) -> Vec<String> {
    let columns = rows.first().map_or(0, Vec::len);
    let widths: Vec<usize> = (0..columns)
        .map(|column| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| char_width(cell))
                .max()
                .unwrap_or(0)
        })
        .collect();
    rows.iter()
        .map(|row| pad_row(row, &widths, right_from))
        .collect()
}

fn summary_label(summary: &Summary, show_mode: bool) -> String {
    let name = variant_label(summary.variant);
    if !show_mode {
        return name;
    }
    let mode = if summary.undo_allowed {
        t("stats.mode.undo")
    } else {
        t("stats.mode.no_undo")
    };
    format!("{name} ({mode})")
}

fn summary_row(summary: &Summary, show_mode: bool, with_clean: bool) -> Vec<String> {
    let mut row = vec![
        summary_label(summary, show_mode),
        summary.played.to_string(),
        summary.won.to_string(),
    ];
    if with_clean {
        row.push(format_count(summary.clean_wins));
    }
    row.extend([
        format!("{}%", summary.win_rate_percent()),
        format_time(summary.best_time_ms),
        format_count(summary.fewest_moves),
        summary.current_streak.to_string(),
        summary.longest_streak.to_string(),
    ]);
    row
}

fn summary_header(with_clean: bool) -> Vec<String> {
    let mut keys = vec![
        "stats.summary.game",
        "stats.summary.played",
        "stats.summary.won",
    ];
    if with_clean {
        keys.push("stats.summary.clean");
    }
    keys.extend([
        "stats.summary.rate",
        "stats.summary.best_time",
        "stats.summary.fewest_moves",
        "stats.summary.streak",
        "stats.summary.longest",
    ]);
    keys.iter().map(|key| t(key).into_owned()).collect()
}

/// The header and the rows of the summary table, padded to one width per column. Wins without a
/// restart get a column beside the wins once a game without undo has finished.
pub fn summary_lines(summaries: &[Summary], show_mode: bool) -> Vec<String> {
    let with_clean = summaries.iter().any(|summary| summary.clean_wins.is_some());
    let mut rows = vec![summary_header(with_clean)];
    rows.extend(
        summaries
            .iter()
            .map(|s| summary_row(s, show_mode, with_clean)),
    );
    table(&rows, 1)
}

/// The widest possible cell of each history column, so that the columns keep their place while
/// the list scrolls: the longest variant name, outcome label, time and count.
fn history_floor() -> Vec<String> {
    let outcomes = [Some(Outcome::Won), Some(Outcome::Abandoned), None].map(outcome_label);
    let proofs = [
        Some(Proof::Winnable),
        Some(Proof::Unwinnable),
        Some(Proof::Unknown),
        None,
    ]
    .map(winnable_label);
    let widest = |texts: Vec<String>| texts.into_iter().max_by_key(|text| char_width(text));
    vec![
        "0000-00-00 00:00".to_owned(),
        widest(Variant::ALL.iter().map(|v| variant_label(*v)).collect()).unwrap_or_default(),
        u64::MAX.to_string(),
        widest(proofs.to_vec()).unwrap_or_default(),
        widest(outcomes.to_vec()).unwrap_or_default(),
        "0:00:00".to_owned(),
        "00000".to_owned(),
        "000".to_owned(),
    ]
}

fn history_header_cells() -> Vec<String> {
    [
        "stats.history.date",
        "stats.history.variant",
        "stats.history.seed",
        "stats.history.winnable",
        "stats.history.outcome",
        "stats.history.time",
        "stats.history.moves",
        "stats.history.restarts",
    ]
    .iter()
    .map(|key| t(key).into_owned())
    .collect()
}

fn history_cells(record: &GameRecord) -> Vec<String> {
    vec![
        format_utc(record.started_at_ms),
        variant_label(record.variant),
        record.seed.to_string(),
        winnable_label(record.proof),
        outcome_label(record.outcome),
        format_time(record.elapsed_ms),
        format_count(record.moves),
        restarts_cell(record),
    ]
}

fn restarts_cell(record: &GameRecord) -> String {
    if record.undo_allowed {
        NONE.to_owned()
    } else {
        record.counters.restarts.to_string()
    }
}

fn history_line(cells: &[String]) -> String {
    let widths: Vec<usize> = history_floor()
        .iter()
        .zip(history_header_cells())
        .map(|(floor, header)| char_width(floor).max(char_width(&header)))
        .collect();
    pad_row(cells, &widths, 5)
}

/// The column header of the history list, without the focus marker columns.
pub fn history_header() -> String {
    history_line(&history_header_cells())
}

/// One history row, without the focus marker columns.
pub fn history_row(record: &GameRecord) -> String {
    history_line(&history_cells(record))
}
