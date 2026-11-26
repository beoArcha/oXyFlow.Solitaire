mod assist;
mod focus;
mod footer;
mod math;
mod render;
mod restarts;
mod results;
mod screen;
mod scrollbar;
mod support;
mod winnable;
mod zone;

use oxysol_core::{DrawCount, Variant};

use super::format::{format_time, history_header};
use crate::glyphs::Glyphs;
use crate::settings::GlyphMode;
use crate::storage::{GameCounters, GameId, GameRecord, Outcome};
use support::history_row;

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
        options: oxysol_core::GameOptions::DEFAULT,
        started_at_ms: 1_700_000_000_000 + id * 60_000,
        finished_at_ms: None,
        outcome,
        elapsed_ms,
        moves,
        undo_allowed: true,
        proof: None,
        counters: GameCounters::default(),
    }
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
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    let won = record(2, KLONDIKE, Some(Outcome::Won), Some(90_000), Some(87));
    let row = history_row(&won, &glyphs, false);
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
    let row = history_row(&open, &glyphs, false);
    assert!(row.contains("In progress"), "{row:?}");
    assert_eq!(
        row.chars().count(),
        history_header(&glyphs, false).chars().count()
    );
}

#[test]
fn rows_and_header_have_one_width() {
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    let long = record(
        7,
        Variant::Pyramid(oxysol_core::PyramidRules::Relaxed),
        Some(Outcome::Abandoned),
        Some(3_725_000),
        Some(123),
    );
    for short in [false, true] {
        assert_eq!(
            history_row(&long, &glyphs, short).chars().count(),
            history_header(&glyphs, short).chars().count()
        );
    }
}
