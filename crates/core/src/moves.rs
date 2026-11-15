//! The command vocabulary shared by every game and the TUI, and the reasons a move is refused.

use crate::pile::PileRef;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A player command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Move {
    /// Draw, deal a row or recycle the waste, depending on the game.
    Stock,
    /// Move the top `count` cards of `from` onto `to`.
    Transfer {
        from: PileRef,
        to: PileRef,
        count: u8,
    },
    /// Pyramid: remove a pair, or a lone king when `second` is `None`.
    Remove {
        first: PileRef,
        second: Option<PileRef>,
    },
}

impl Move {
    /// The pile the cards are taken from, if the move has one.
    pub fn source(&self) -> Option<PileRef> {
        match self {
            Move::Stock => None,
            Move::Transfer { from, .. } => Some(*from),
            Move::Remove { first, .. } => Some(*first),
        }
    }

    /// The pile the cards are put on, if the move has one.
    pub fn target(&self) -> Option<PileRef> {
        match self {
            Move::Stock | Move::Remove { second: None, .. } => None,
            Move::Transfer { to, .. } => Some(*to),
            Move::Remove { second, .. } => *second,
        }
    }
}

/// Why a move was rejected. An expected validation result, never a system error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MoveError {
    #[error("this move is not supported by the game")]
    Unsupported,
    #[error("pile {0:?} does not exist in this game")]
    NoSuchPile(PileRef),
    #[error("pile {0:?} has no cards to move")]
    EmptySource(PileRef),
    #[error("the cards are face down")]
    FaceDown,
    #[error("the cards do not form a movable sequence")]
    InvalidSequence,
    #[error("the target pile does not accept these cards")]
    TargetRejects,
    #[error("the stock is empty")]
    StockEmpty,
    #[error("a row cannot be dealt while a column is empty")]
    EmptyColumnBlocksDeal,
    #[error("no more passes through the stock are allowed")]
    PassesExhausted,
    #[error("{count} cards cannot be moved here")]
    CountOutOfRange { count: u8 },
    #[error("there is not enough free space to move the sequence")]
    NotEnoughSpace,
    #[error("the cards do not form a removable pair")]
    NotAPair,
    #[error("the card is covered by other cards")]
    Blocked,
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM: PileRef = PileRef::Waste;
    const TO: PileRef = PileRef::Tableau(2);

    #[test]
    fn source_and_target() {
        let transfer = Move::Transfer {
            from: FROM,
            to: TO,
            count: 1,
        };
        assert_eq!(Move::Stock.source(), None);
        assert_eq!(Move::Stock.target(), None);
        assert_eq!(transfer.source(), Some(FROM));
        assert_eq!(transfer.target(), Some(TO));
        let pair = Move::Remove {
            first: FROM,
            second: Some(TO),
        };
        assert_eq!(pair.source(), Some(FROM));
        assert_eq!(pair.target(), Some(TO));
        let king = Move::Remove {
            first: FROM,
            second: None,
        };
        assert_eq!(king.source(), Some(FROM));
        assert_eq!(king.target(), None);
    }

    #[test]
    fn move_json_is_stable() {
        let cases = [
            (Move::Stock, r#""stock""#),
            (
                Move::Transfer {
                    from: FROM,
                    to: TO,
                    count: 3,
                },
                r#"{"transfer":{"from":"waste","to":{"tableau":2},"count":3}}"#,
            ),
            (
                Move::Remove {
                    first: FROM,
                    second: None,
                },
                r#"{"remove":{"first":"waste","second":null}}"#,
            ),
        ];
        for (mv, json) in cases {
            assert_eq!(serde_json::to_string(&mv).unwrap(), json);
            assert_eq!(serde_json::from_str::<Move>(json).unwrap(), mv);
        }
    }

    #[test]
    fn errors_have_messages() {
        assert_eq!(MoveError::StockEmpty.to_string(), "the stock is empty");
        assert!(
            MoveError::EmptyColumnBlocksDeal
                .to_string()
                .contains("column is empty")
        );
        assert!(MoveError::EmptySource(FROM).to_string().contains("Waste"));
    }
}
