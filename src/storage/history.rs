//! The game history: every dealt seed with its variant, times and outcome (R-18).

use oxysol_tui::{AssistCounters, GameCounters, GameId, GameRecord};
use rusqlite::{Connection, Row};

use super::convert::{
    count_from_column, millis_from_column, options_from_column, outcome_from_column,
    proof_from_column, seed_from_column, variant_from_column,
};
use super::error::Fault;

struct RawRow {
    id: i64,
    variant: String,
    seed: i64,
    started_at: i64,
    finished_at: Option<i64>,
    outcome: Option<String>,
    elapsed_ms: Option<i64>,
    moves: Option<i64>,
    undo_allowed: i64,
    proof: Option<String>,
    restarts: i64,
    assist: [i64; 5],
    options: String,
}

impl RawRow {
    fn read(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            variant: row.get(1)?,
            seed: row.get(2)?,
            started_at: row.get(3)?,
            finished_at: row.get(4)?,
            outcome: row.get(5)?,
            elapsed_ms: row.get(6)?,
            moves: row.get(7)?,
            undo_allowed: row.get(8)?,
            proof: row.get(9)?,
            restarts: row.get(10)?,
            assist: [
                row.get(11)?,
                row.get(12)?,
                row.get(13)?,
                row.get(14)?,
                row.get(15)?,
            ],
            options: row.get(16)?,
        })
    }

    fn into_record(self) -> Result<GameRecord, Fault> {
        Ok(GameRecord {
            id: GameId(self.id),
            variant: variant_from_column(&self.variant)?,
            seed: seed_from_column(self.seed),
            started_at_ms: self.started_at,
            finished_at_ms: self.finished_at,
            outcome: self
                .outcome
                .as_deref()
                .map(outcome_from_column)
                .transpose()?,
            elapsed_ms: self.elapsed_ms.map(millis_from_column).transpose()?,
            moves: self.moves.map(count_from_column).transpose()?,
            options: options_from_column(&self.options)?,
            undo_allowed: self.undo_allowed != 0,
            proof: self.proof.as_deref().map(proof_from_column).transpose()?,
            counters: GameCounters {
                restarts: count_from_column(self.restarts)?,
                assist: AssistCounters {
                    manual: count_from_column(self.assist[0])?,
                    quick: count_from_column(self.assist[1])?,
                    finish: count_from_column(self.assist[2])?,
                    failed: count_from_column(self.assist[3])?,
                    longest_streak: count_from_column(self.assist[4])?,
                },
            },
        })
    }
}

pub(super) fn read_all(conn: &Connection) -> Result<Vec<GameRecord>, Fault> {
    let mut statement = conn.prepare(
        "SELECT id, variant, seed, started_at, finished_at, outcome, elapsed_ms, moves, undo_allowed,
                proof, restarts, manual_moves, quick_moves, finish_moves, failed_quick, quick_streak,
                options
         FROM games ORDER BY id",
    )?;
    let rows = statement.query_map([], RawRow::read)?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?.into_record()?);
    }
    Ok(records)
}
