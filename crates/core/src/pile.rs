//! Piles: how games and the TUI name places on the table and view their contents.

use crate::card::Card;
use serde::{Deserialize, Serialize};

/// A place on the table. Not every game uses every kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PileRef {
    Stock,
    Waste,
    Foundation(u8),
    Tableau(u8),
    Cell(u8),
    Pyramid { row: u8, index: u8 },
}

/// A card on the table together with its orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCard {
    pub card: Card,
    pub face_up: bool,
}

/// Read-only view of one pile, cards listed from bottom to top.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PileSnapshot {
    pub pile: PileRef,
    pub cards: Vec<TableCard>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardId, Rank, Suit};

    #[test]
    fn pile_ref_json_is_stable() {
        let cases = [
            (PileRef::Stock, r#""stock""#),
            (PileRef::Waste, r#""waste""#),
            (PileRef::Foundation(2), r#"{"foundation":2}"#),
            (PileRef::Tableau(6), r#"{"tableau":6}"#),
            (PileRef::Cell(1), r#"{"cell":1}"#),
            (
                PileRef::Pyramid { row: 3, index: 1 },
                r#"{"pyramid":{"row":3,"index":1}}"#,
            ),
        ];
        for (pile, json) in cases {
            assert_eq!(serde_json::to_string(&pile).unwrap(), json);
            assert_eq!(serde_json::from_str::<PileRef>(json).unwrap(), pile);
        }
    }

    #[test]
    fn snapshot_json_is_stable() {
        let card = Card {
            id: CardId(5),
            suit: Suit::Hearts,
            rank: Rank::Queen,
        };
        let snapshot = PileSnapshot {
            pile: PileRef::Tableau(0),
            cards: vec![TableCard {
                card,
                face_up: true,
            }],
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert_eq!(
            json,
            r#"{"pile":{"tableau":0},"cards":[{"card":{"id":5,"suit":"hearts","rank":"queen"},"face_up":true}]}"#
        );
        assert_eq!(
            serde_json::from_str::<PileSnapshot>(&json).unwrap(),
            snapshot
        );
    }
}
