//! The contract between the engine and the TUI: the [`Game`] trait, the playable [`Variant`]s and
//! [`deal`], the only place that knows every game.

use crate::games::freecell::FreeCell;
use crate::games::klondike::Klondike;
use crate::games::pyramid::Pyramid;
use crate::games::spider::Spider;
use crate::moves::{Move, MoveError};
use crate::pile::PileSnapshot;
use serde::{Deserialize, Serialize};

/// Cards turned over from the stock at once in Klondike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawCount {
    One,
    Three,
}

/// Number of suits in a Spider deal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuitCount {
    One,
    Two,
    Four,
}

/// Pyramid rule set (R-11): `Relaxed` allows 3 passes through the stock and is won when the pyramid
/// is empty; `Classic` allows 1 pass and is won when all 52 cards are removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PyramidRules {
    Relaxed,
    Classic,
}

/// A playable game with its options. Serialized with stable snake_case names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Klondike(DrawCount),
    Spider(SuitCount),
    FreeCell,
    Pyramid(PyramidRules),
}

impl Variant {
    /// Every concrete variant the menu can offer.
    pub const ALL: [Variant; 8] = [
        Variant::Klondike(DrawCount::One),
        Variant::Klondike(DrawCount::Three),
        Variant::Spider(SuitCount::One),
        Variant::Spider(SuitCount::Two),
        Variant::Spider(SuitCount::Four),
        Variant::FreeCell,
        Variant::Pyramid(PyramidRules::Relaxed),
        Variant::Pyramid(PyramidRules::Classic),
    ];
}

/// One running game. The trait has no supertraits: the single-threaded TUI needs neither `Send`
/// nor `Debug`, and a bound can be added later without touching the games.
pub trait Game {
    /// Validates, executes and performs automatic effects. On `Err` the state is unchanged.
    fn apply(&mut self, mv: Move) -> Result<(), MoveError>;

    /// Exactly the moves that [`Game::apply`] accepts.
    fn legal_moves(&self) -> Vec<Move>;

    /// A suggested move, always one of [`Game::legal_moves`].
    fn hint(&self) -> Option<Move>;

    /// Whether the game is won.
    fn is_won(&self) -> bool;

    /// Read-only view of every pile, cards bottom to top.
    fn piles(&self) -> Vec<PileSnapshot>;
}

/// Deals `variant` from `seed`. The same pair always gives the same deal.
pub fn deal(variant: Variant, seed: u64) -> Box<dyn Game> {
    match variant {
        Variant::Klondike(draw) => Box::new(Klondike::deal(draw, seed)),
        Variant::Spider(suits) => Box::new(Spider::deal(suits, seed)),
        Variant::FreeCell => Box::new(FreeCell::deal(seed)),
        Variant::Pyramid(rules) => Box::new(Pyramid::deal(rules, seed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn card_ids(game: &dyn Game) -> Vec<u16> {
        let mut ids: Vec<u16> = game
            .piles()
            .iter()
            .flat_map(|pile| pile.cards.iter().map(|c| c.card.id.0))
            .collect();
        ids.sort_unstable();
        ids
    }

    #[test]
    fn all_lists_eight_distinct_variants() {
        let unique: HashSet<_> = Variant::ALL.iter().collect();
        assert_eq!(unique.len(), 8);
    }

    #[test]
    fn variant_json_is_stable() {
        let cases = [
            (Variant::Klondike(DrawCount::One), r#"{"klondike":"one"}"#),
            (
                Variant::Klondike(DrawCount::Three),
                r#"{"klondike":"three"}"#,
            ),
            (Variant::Spider(SuitCount::One), r#"{"spider":"one"}"#),
            (Variant::Spider(SuitCount::Two), r#"{"spider":"two"}"#),
            (Variant::Spider(SuitCount::Four), r#"{"spider":"four"}"#),
            (Variant::FreeCell, r#""free_cell""#),
            (
                Variant::Pyramid(PyramidRules::Relaxed),
                r#"{"pyramid":"relaxed"}"#,
            ),
            (
                Variant::Pyramid(PyramidRules::Classic),
                r#"{"pyramid":"classic"}"#,
            ),
        ];
        assert_eq!(cases.len(), Variant::ALL.len());
        for (variant, json) in cases {
            assert_eq!(serde_json::to_string(&variant).unwrap(), json);
            assert_eq!(serde_json::from_str::<Variant>(json).unwrap(), variant);
        }
    }

    #[test]
    fn deals_have_the_right_cards_with_unique_ids() {
        for variant in Variant::ALL {
            let expected = match variant {
                Variant::Spider(_) => 104,
                _ => 52,
            };
            let ids = card_ids(deal(variant, 7).as_ref());
            assert_eq!(ids, (0..expected).collect::<Vec<u16>>(), "{variant:?}");
        }
    }

    #[test]
    fn deal_is_deterministic_and_depends_on_the_seed() {
        for variant in Variant::ALL {
            let first = deal(variant, 42).piles();
            assert_eq!(first, deal(variant, 42).piles(), "{variant:?}");
            assert_ne!(first, deal(variant, 43).piles(), "{variant:?}");
        }
    }
}
