//! Text of the statistics: dates, durations and the rows of the two tables.

use std::time::Duration;

use super::summary::Summary;
use crate::clock::format_elapsed;
use crate::glyphs::Glyphs;
use crate::i18n::t;
use crate::screens::variant_label_key;
use crate::solver::Proof;
use crate::storage::{GameRecord, Outcome};
use crate::theme::Token;
use oxysol_core::Variant;

const NONE: &str = "-";
/// The first column that is right-aligned: the counts and times, not the texts.
const FIRST_RIGHT_COLUMN: usize = 5;
const MILLIS_PER_DAY: i64 = 86_400_000;
const MILLIS_PER_MINUTE: i64 = 60_000;
const COLUMN_GAP: &str = " ";

/// Year, month and day of a day count since 1970-01-01 in the proleptic Gregorian calendar.
fn civil_date(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let doe = shifted.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mi = (5 * doy + 2) / 153;
    let day = doy - (153 * mi + 2) / 5 + 1;
    let mo = if mi < 10 { mi + 3 } else { mi - 9 };
    (yoe + era * 400 + i64::from(mo <= 2), mo, day)
}

/// `YYYY-MM-DD HH:MM` in UTC for Unix milliseconds. The standard library cannot convert to the
/// local time zone, so the history is shown in UTC and its heading says so.
pub fn format_utc(unix_ms: i64) -> String {
    let days = unix_ms.div_euclid(MILLIS_PER_DAY);
    let m = unix_ms.rem_euclid(MILLIS_PER_DAY) / MILLIS_PER_MINUTE;
    let (y, mo, d) = civil_date(days);
    format!("{y:04}-{mo:02}-{d:02} {:02}:{:02}", m / 60, m % 60)
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

fn winnable_label(proof: Option<Proof>) -> &'static str {
    match proof {
        Some(Proof::Winnable) => "stats.winnable.yes",
        Some(Proof::Unwinnable) => "stats.winnable.no",
        Some(Proof::Unknown) => "stats.winnable.unknown",
        None => "stats.legend.unchecked",
    }
}

fn winnable_cell(proof: Option<Proof>, glyphs: &Glyphs, short: bool) -> String {
    let mark = glyphs.proof_mark(proof);
    if short {
        mark.to_owned()
    } else {
        format!("{mark} {}", t(winnable_label(proof)))
    }
}

/// The one-line legend under the history list, explaining the winnable signs.
pub fn legend_line(glyphs: &Glyphs) -> String {
    let items = [
        (glyphs.proof_winnable, t("stats.legend.winnable")),
        (glyphs.proof_unwinnable, t("stats.legend.unwinnable")),
        (glyphs.proof_unknown, t("stats.legend.unknown")),
        (glyphs.proof_unchecked, t("stats.legend.unchecked")),
    ];
    items
        .into_iter()
        .map(|(glyph, text)| format!("{glyph} {text}"))
        .collect::<Vec<_>>()
        .join("   ")
}

fn variant_label(variant: Variant) -> String {
    t(variant_label_key(variant)).into_owned()
}

fn char_width(text: &str) -> usize {
    text.chars().count()
}

fn pad_cells(cells: &[String], widths: &[usize], right_from: usize) -> Vec<String> {
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
        .collect()
}

fn pad_row(cells: &[String], widths: &[usize], right_from: usize) -> String {
    pad_cells(cells, widths, right_from).join(COLUMN_GAP)
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
        summary.assisted.to_string(),
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
        "stats.summary.assisted",
    ]);
    keys.into_iter().map(|key| t(key).into_owned()).collect()
}

/// The narrowest terminal that has room for the longest-streak and assisted columns of the summary.
pub const LONGEST_STREAK_MIN_WIDTH: u16 = 110;
/// The narrowest terminal that has room for the winnable text column beside the mark.
pub const WINNABLE_TEXT_MIN_WIDTH: u16 = 110;

/// The header and the rows of the summary table, padded to one width per column. Wins without a
/// restart get a column beside the wins once a game without undo has finished. The longest-streak
/// column is left out unless `with_longest`.
pub fn summary_lines(summaries: &[Summary], show_mode: bool, with_longest: bool) -> Vec<String> {
    let with_clean = summaries.iter().any(|summary| summary.clean_wins.is_some());
    let mut rows = vec![summary_header(with_clean)];
    rows.extend(
        summaries
            .iter()
            .map(|s| summary_row(s, show_mode, with_clean)),
    );
    if !with_longest {
        for row in &mut rows {
            row.truncate(row.len().saturating_sub(2));
        }
    }
    table(&rows, 1)
}

/// The widest possible cell of each history column, so that the columns keep their place while
/// the list scrolls: the longest variant name, outcome label, time and count.
fn history_floor(glyphs: &Glyphs, short: bool) -> Vec<String> {
    let outcomes = [Some(Outcome::Won), Some(Outcome::Abandoned), None].map(outcome_label);
    let proofs = [
        Some(Proof::Winnable),
        Some(Proof::Unwinnable),
        Some(Proof::Unknown),
        None,
    ]
    .map(|p| winnable_cell(p, glyphs, short));
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

fn history_header_cells(short: bool) -> Vec<String> {
    let winnable = if short {
        "stats.history.winnable_short"
    } else {
        "stats.history.winnable"
    };
    [
        "stats.history.date",
        "stats.history.variant",
        "stats.history.seed",
        winnable,
        "stats.history.outcome",
        "stats.history.time",
        "stats.history.moves",
        "stats.history.restarts",
    ]
    .into_iter()
    .map(|key| t(key).into_owned())
    .collect()
}

fn history_cells(record: &GameRecord, glyphs: &Glyphs, short: bool) -> Vec<String> {
    vec![
        format_utc(record.started_at_ms),
        variant_label(record.variant),
        record.seed.to_string(),
        winnable_cell(record.proof, glyphs, short),
        outcome_label(record.outcome),
        format_time(record.elapsed_ms),
        format_count(record.moves),
        restarts_cell(record),
    ]
}

fn restarts_cell(record: &GameRecord) -> String {
    match record.undo_allowed {
        true => NONE.to_owned(),
        false => record.counters.restarts.to_string(),
    }
}

/// The index of the result column in the history list.
pub const RESULT_COLUMN: usize = 4;

fn history_widths(glyphs: &Glyphs, short: bool) -> Vec<usize> {
    history_floor(glyphs, short)
        .iter()
        .zip(history_header_cells(short))
        .map(|(floor, header)| char_width(floor).max(char_width(&header)))
        .collect()
}

fn history_line(cells: &[String], glyphs: &Glyphs, short: bool) -> String {
    pad_row(cells, &history_widths(glyphs, short), FIRST_RIGHT_COLUMN)
}

/// The column header of the history list, without the focus marker columns.
pub fn history_header(glyphs: &Glyphs, short: bool) -> String {
    history_line(&history_header_cells(short), glyphs, short)
}

/// The cells of one history row, each padded to its column, without the focus marker columns. The
/// screen styles the [`RESULT_COLUMN`] cell by the result.
pub fn history_columns(record: &GameRecord, glyphs: &Glyphs, short: bool) -> Vec<String> {
    pad_cells(
        &history_cells(record, glyphs, short),
        &history_widths(glyphs, short),
        FIRST_RIGHT_COLUMN,
    )
}

/// The colour token of a result: won reads as success, abandoned as muted, a game in progress as
/// the accent. The result word itself carries the meaning too.
pub const fn result_token(outcome: Option<Outcome>) -> Token {
    match outcome {
        Some(Outcome::Won) => Token::Success,
        Some(Outcome::Abandoned) => Token::TextMuted,
        None => Token::Accent,
    }
}
