mod math;
mod render;
mod screen;
mod support;

use oxysol_core::{DrawCount, Variant};

use super::format::{format_time, format_utc, history_header, history_row};
use crate::storage::{GameId, GameRecord, Outcome};

pub const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

pub fn record(
    id: i64,
    variant: Variant,
    outcome: Option<Outcome>,
    elapsed_ms: Option<u64>,
    moves: Option<u32>,
) -> GameRecord {
    GameRecord {
        id: GameId(id),
        variant,
        seed: u64::try_from(id).unwrap_or(0) * 11,
        started_at_ms: 1_700_000_000_000 + id * 60_000,
        finished_at_ms: None,
        outcome,
        elapsed_ms,
        moves,
        undo_allowed: true,
    }
}

#[test]
fn utc_dates_are_formatted_without_a_dependency() {
    assert_eq!(format_utc(0), "1970-01-01 00:00");
    assert_eq!(format_utc(1_700_000_000_000), "2023-11-14 22:13");
    assert_eq!(format_utc(951_782_400_000), "2000-02-29 00:00");
    assert_eq!(format_utc(-1), "1969-12-31 23:59");
    assert_eq!(format_utc(4_102_444_799_000), "2099-12-31 23:59");
}

#[test]
fn times_are_m_ss_then_h_mm_ss_and_unknown_is_a_dash() {
    assert_eq!(format_time(Some(0)), "0:00");
    assert_eq!(format_time(Some(65_000)), "1:05");
    assert_eq!(format_time(Some(3_725_000)), "1:02:05");
    assert_eq!(format_time(None), "-");
}

#[test]
fn a_history_row_lists_date_game_seed_result_time_and_moves() {
    let won = record(2, KLONDIKE, Some(Outcome::Won), Some(90_000), Some(87));
    let row = history_row(&won);
    for part in [
        "2023-11-14 22:15",
        "Klondike, Draw 1",
        "22",
        "Won",
        "1:30",
        "87",
    ] {
        assert!(row.contains(part), "{row:?} lacks {part:?}");
    }
    let open = record(3, KLONDIKE, None, None, None);
    let row = history_row(&open);
    assert!(row.contains("In progress"), "{row:?}");
    assert_eq!(row.chars().count(), history_header().chars().count());
}

#[test]
fn rows_and_header_have_one_width() {
    let long = record(
        7,
        Variant::Pyramid(oxysol_core::PyramidRules::Relaxed),
        Some(Outcome::Abandoned),
        Some(3_725_000),
        Some(123),
    );
    assert_eq!(
        history_row(&long).chars().count(),
        history_header().chars().count()
    );
}
