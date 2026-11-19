//! Statistics over the game history: one [`Summary`] per variant and undo mode. Pure functions
//! on [`GameRecord`]s, so every number is testable with a fixed history.

use oxysol_core::Variant;

use crate::storage::{GameRecord, Outcome};

/// The numbers of one variant played in one undo mode. Only finished games count: a game still
/// open is neither played nor lost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub variant: Variant,
    pub undo_allowed: bool,
    pub played: u32,
    pub won: u32,
    /// Wins without a restart; `None` with undo, where restarts are not counted (P4-05).
    pub clean_wins: Option<u32>,
    /// Shortest playing time of a win, milliseconds.
    pub best_time_ms: Option<u64>,
    /// Fewest moves in a win; wins recorded before moves were counted are skipped.
    pub fewest_moves: Option<u32>,
    /// Wins in a row at the end of the history; an abandoned game ends the streak.
    pub current_streak: u32,
    pub longest_streak: u32,
}

impl Summary {
    /// Share of won games in percent, rounded to the nearest whole number.
    pub fn win_rate_percent(&self) -> u32 {
        if self.played == 0 {
            return 0;
        }
        let played = u64::from(self.played);
        let rate = (u64::from(self.won) * 100 + played / 2) / played;
        u32::try_from(rate).unwrap_or(100)
    }
}

fn is_won(record: &GameRecord) -> bool {
    record.outcome == Some(Outcome::Won)
}

fn streaks(group: &[&GameRecord]) -> (u32, u32) {
    let (mut current, mut longest) = (0_u32, 0_u32);
    for record in group {
        if is_won(record) {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    (current, longest)
}

fn count(group: &[&GameRecord], keep: impl Fn(&GameRecord) -> bool) -> u32 {
    u32::try_from(group.iter().filter(|record| keep(record)).count()).unwrap_or(u32::MAX)
}

fn is_clean_win(record: &GameRecord) -> bool {
    is_won(record) && record.counters.restarts == 0
}

fn summarize_group(variant: Variant, undo_allowed: bool, group: &[&GameRecord]) -> Summary {
    let wins = group.iter().copied().filter(|record| is_won(record));
    let (current_streak, longest_streak) = streaks(group);
    Summary {
        variant,
        undo_allowed,
        played: count(group, |_| true),
        won: count(group, is_won),
        clean_wins: (!undo_allowed).then(|| count(group, is_clean_win)),
        best_time_ms: wins.clone().filter_map(|record| record.elapsed_ms).min(),
        fewest_moves: wins.filter_map(|record| record.moves).min(),
        current_streak,
        longest_streak,
    }
}

/// One summary per variant and undo mode that has a finished game, in menu order (undo first).
/// Streaks follow the order in which the games were dealt, whatever the order of `records`.
pub fn summarize(records: &[GameRecord]) -> Vec<Summary> {
    let mut finished: Vec<&GameRecord> = records
        .iter()
        .filter(|record| record.outcome.is_some())
        .collect();
    finished.sort_by_key(|record| record.id.0);
    let mut summaries = Vec::new();
    for variant in Variant::ALL {
        for undo_allowed in [true, false] {
            let group: Vec<&GameRecord> = finished
                .iter()
                .copied()
                .filter(|record| record.variant == variant && record.undo_allowed == undo_allowed)
                .collect();
            if !group.is_empty() {
                summaries.push(summarize_group(variant, undo_allowed, &group));
            }
        }
    }
    summaries
}

/// Whether the history holds more than one undo mode, so that the summary names the mode.
pub fn has_several_modes(records: &[GameRecord]) -> bool {
    records.iter().any(|record| record.undo_allowed)
        && records.iter().any(|record| !record.undo_allowed)
}
