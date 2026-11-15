//! Localized messages of the board status line.

use oxysol_core::MoveError;

/// Localization key of the message for a rejected move. Every [`MoveError`] has one; the match is
/// exhaustive on purpose, so a new variant cannot be forgotten.
pub(super) const fn error_key(error: &MoveError) -> &'static str {
    match error {
        MoveError::Unsupported => "board.error.unsupported",
        MoveError::NoSuchPile(_) => "board.error.no_such_pile",
        MoveError::EmptySource(_) => "board.error.empty_source",
        MoveError::FaceDown => "board.error.face_down",
        MoveError::InvalidSequence => "board.error.invalid_sequence",
        MoveError::TargetRejects => "board.error.target_rejects",
        MoveError::StockEmpty => "board.error.stock_empty",
        MoveError::EmptyColumnBlocksDeal => "board.error.empty_column_blocks_deal",
        MoveError::PassesExhausted => "board.error.passes_exhausted",
        MoveError::CountOutOfRange { .. } => "board.error.count_out_of_range",
        MoveError::NotEnoughSpace => "board.error.not_enough_space",
        MoveError::NotAPair => "board.error.not_a_pair",
        MoveError::Blocked => "board.error.blocked",
    }
}

pub(super) const PICKED: &str = "board.status.picked";
pub(super) const NO_TARGETS: &str = "board.status.no_targets";
pub(super) const NOTHING_TO_UNDO: &str = "board.status.nothing_to_undo";
pub(super) const NOTHING_TO_REDO: &str = "board.status.nothing_to_redo";
pub(super) const NO_HINT: &str = "board.status.no_hint";
pub(super) const HINT: &str = "board.status.hint";
pub(super) const HINT_STOCK: &str = "board.status.hint_stock";
pub(super) const NO_MOVES: &str = "board.status.no_moves";
pub(super) const SOLVED: &str = "board.status.solved";
