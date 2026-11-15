//! The game history: every dealt seed with its variant, times and outcome (R-18).

use oxysol_core::Variant;
use oxysol_tui::{GameId, Outcome};
use rusqlite::{Connection, Row};

use super::convert::{
    millis_from_column, outcome_from_column, seed_from_column, variant_from_column,
};
use super::error::Fault;

/// One dealt game as recorded in the history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRecord {
    pub id: GameId,
    pub variant: Variant,
    pub seed: u64,
    /// UTC, Unix milliseconds.
    pub started_at_ms: i64,
    /// UTC, Unix milliseconds; `None` while the game is open.
    pub finished_at_ms: Option<i64>,
    pub outcome: Option<Outcome>,
    pub elapsed_ms: Option<u64>,
}

struct RawRow {
    id: i64,
    variant: String,
    seed: i64,
    started_at: i64,
    finished_at: Option<i64>,
    outcome: Option<String>,
    elapsed_ms: Option<i64>,
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
        })
    }
}

pub(super) fn read_all(conn: &Connection) -> Result<Vec<GameRecord>, Fault> {
    let mut statement = conn.prepare(
        "SELECT id, variant, seed, started_at, finished_at, outcome, elapsed_ms
         FROM games ORDER BY id",
    )?;
    let rows = statement.query_map([], RawRow::read)?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?.into_record()?);
    }
    Ok(records)
}
