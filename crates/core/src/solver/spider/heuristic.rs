//! Heuristic evaluation and dead-end detection for 2- and 4-suit Spider positions.

use super::position::{COLUMNS, Position};
use crate::solver::face::{KING, rank, suit};

const DECK_CARDS: u32 = 104;
const CARDS_PER_RUN: u32 = 13;
const EMPTY_BONUSES: [u32; 5] = [0, 26, 44, 56, 62];
const MAX_EMPTY_BONUS: u32 = 62;

struct ColumnMetrics {
    runs: u32,
    inversions: u32,
    gaps: u32,
    soft_breaks: u32,
    clean_king_cards: u32,
    buried_runs: u32,
}

fn scan_column(pos: &Position, column: usize) -> ColumnMetrics {
    let up = &pos.cols[column][pos.hidden[column]..pos.len[column]];
    if up.is_empty() {
        return ColumnMetrics {
            runs: 0,
            inversions: 0,
            gaps: 0,
            soft_breaks: 0,
            clean_king_cards: 0,
            buried_runs: 0,
        };
    }
    let mut runs = 1u32;
    let mut inversions = 0u32;
    let mut gaps = 0u32;
    let mut soft_breaks = 0u32;
    let mut leading_contiguous = true;
    let mut leading_len = 1u32;
    for pair in up.windows(2) {
        let (upper, lower) = (pair[0], pair[1]);
        let same_suit_step = rank(upper) == rank(lower) + 1 && suit(upper) == suit(lower);
        if same_suit_step {
            if leading_contiguous {
                leading_len += 1;
            }
            continue;
        }
        leading_contiguous = false;
        runs += 1;
        if rank(upper) <= rank(lower) {
            inversions += 1;
        } else if rank(upper) > rank(lower) + 1 {
            gaps += 1;
        } else {
            soft_breaks += 1;
        }
    }
    let clean_king_cards = if pos.hidden[column] == 0 && rank(up[0]) == KING {
        leading_len
    } else {
        0
    };
    let buried_runs = if pos.hidden[column] > 0 { runs } else { 0 };
    ColumnMetrics {
        runs,
        inversions,
        gaps,
        soft_breaks,
        clean_king_cards,
        buried_runs,
    }
}

impl Position {
    /// True when the stock still holds cards that can never be dealt because fewer than ten cards
    /// remain on the tableau.
    pub(super) fn is_dead(&self) -> bool {
        let tableau_cards: usize = self.len.iter().sum();
        self.stock_len > 0 && tableau_cards < COLUMNS
    }

    /// Estimated work remaining (lower is closer to a win; zero at a won position).
    pub(super) fn work_left(&self) -> u32 {
        let cards_left = DECK_CARDS - CARDS_PER_RUN * u32::from(self.removed);
        let hidden_total: usize = self.hidden.iter().sum();
        let empty = self.len.iter().filter(|&&len| len == 0).count();
        let stock_len = u32::try_from(self.stock_len).unwrap_or(0);
        let mut runs = 0u32;
        let mut inversions = 0u32;
        let mut gaps = 0u32;
        let mut soft_breaks = 0u32;
        let mut clean_king_cards = 0u32;
        let mut buried_runs = 0u32;
        let mut face_up_total = 0u32;
        for column in 0..COLUMNS {
            face_up_total += u32::try_from(self.face_up(column)).unwrap_or(0);
            let m = scan_column(self, column);
            runs += m.runs;
            inversions += m.inversions;
            gaps += m.gaps;
            soft_breaks += m.soft_breaks;
            clean_king_cards += m.clean_king_cards;
            buried_runs += m.buried_runs;
        }
        let empty_penalty = MAX_EMPTY_BONUS - EMPTY_BONUSES[empty.min(EMPTY_BONUSES.len() - 1)];
        let non_king_up = face_up_total.saturating_sub(clean_king_cards);
        let hidden_u32 = u32::try_from(hidden_total).unwrap_or(0);
        16 * cards_left
            + 26 * hidden_u32
            + 22 * runs
            + 27 * stock_len
            + 10 * inversions
            + 6 * gaps
            + 2 * soft_breaks
            + 3 * buried_runs
            + 3 * non_king_up
            + empty_penalty
    }
}
