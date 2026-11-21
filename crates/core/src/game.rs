//! The contract between the engine and the TUI: the [`Game`] trait, the playable [`Variant`]s and
//! [`deal`], the only place that knows every game.

use crate::games::freecell::FreeCell;
use crate::games::klondike::Klondike;
use crate::games::pyramid::Pyramid;
use crate::games::spider::Spider;
use crate::games::yukon::Yukon;
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

/// Rule set of the Yukon family (P4-10a). Every rule set shares the Yukon group moves; it chooses
/// only how columns are built and how the cards are dealt (`games::yukon::setup`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YukonRules {
    /// Classic Yukon: columns build down in alternating colours; 21 face-down, 31 face-up cards.
    Classic,
    /// Russian Solitaire: the classic deal; columns build downward in the same suit.
    Russian,
    /// Alaska: the classic deal; columns build up or down by one rank in the same suit.
    Alaska,
    /// Moosehide: the classic deal; columns build down by one rank in any suit but the card's own.
    Moosehide,
    /// Australian Patience: seven columns of four face-up cards, a stock turned once to a waste;
    /// columns build downward in the same suit.
    Australian,
}

/// The state of a game's stock, read-only. A recycle turns the waste back into the stock and
/// completes one pass. The first traversal is pass 1 and is in progress until its recycle, so the
/// pass in progress is `recycles + 1`. Pyramid's "3 passes" (Relaxed) are the first traversal plus
/// 2 recycles, so its `recycles_left` is 2 right after the deal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StockStatus {
    /// Recycles made so far: 0 right after the deal, +1 for each waste turned back into the stock.
    /// A rejected recycle does not count. Spider never recycles, so it is always 0.
    pub recycles: u32,
    /// Recycles still allowed. `None` when the stock is unlimited (Klondike). `Some(0)` when no
    /// recycle can happen at all (Spider) or the last allowed one has been made (Pyramid).
    pub recycles_left: Option<u32>,
    /// Spider only: the row deals still in the stock, one per 10 cards. `None` for other games.
    pub deals_left: Option<u32>,
}

/// A playable game with its options. Serialized with stable snake_case names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Klondike(DrawCount),
    Spider(SuitCount),
    FreeCell,
    Pyramid(PyramidRules),
    Yukon(YukonRules),
}

impl Variant {
    /// Every concrete variant the menu can offer.
    pub const ALL: [Variant; 13] = [
        Variant::Klondike(DrawCount::One),
        Variant::Klondike(DrawCount::Three),
        Variant::Spider(SuitCount::One),
        Variant::Spider(SuitCount::Two),
        Variant::Spider(SuitCount::Four),
        Variant::FreeCell,
        Variant::Pyramid(PyramidRules::Relaxed),
        Variant::Pyramid(PyramidRules::Classic),
        Variant::Yukon(YukonRules::Classic),
        Variant::Yukon(YukonRules::Russian),
        Variant::Yukon(YukonRules::Alaska),
        Variant::Yukon(YukonRules::Moosehide),
        Variant::Yukon(YukonRules::Australian),
    ];

    /// Whether the variant deals part of the deck to a stock: every game except FreeCell and the
    /// Yukon family, where only Australian Patience has one.
    pub const fn has_stock(self) -> bool {
        match self {
            Variant::FreeCell => false,
            Variant::Yukon(rules) => matches!(rules, YukonRules::Australian),
            _ => true,
        }
    }
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

    /// The stock counters, or `None` for a game without a stock (FreeCell).
    fn stock_status(&self) -> Option<StockStatus>;

    /// The moves that finish the game when the rest of it is a certain win, otherwise `None`.
    /// Applied one after another through [`Game::apply`] they are all legal and the last one makes
    /// [`Game::is_won`] true. Pure and deterministic: a fixed greedy rule per game, never a search
    /// beyond a small documented bound, and every sequence is verified on a clone before it is
    /// returned. `None` also for a game that is already won.
    fn forced_finish(&self) -> Option<Vec<Move>>;
}

/// Deals `variant` from `seed`. The same pair always gives the same deal.
pub fn deal(variant: Variant, seed: u64) -> Box<dyn Game> {
    match variant {
        Variant::Klondike(draw) => Box::new(Klondike::deal(draw, seed)),
        Variant::Spider(suits) => Box::new(Spider::deal(suits, seed)),
        Variant::FreeCell => Box::new(FreeCell::deal(seed)),
        Variant::Pyramid(rules) => Box::new(Pyramid::deal(rules, seed)),
        Variant::Yukon(rules) => Box::new(Yukon::deal(rules, seed)),
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
    fn all_lists_thirteen_distinct_variants() {
        let unique: HashSet<_> = Variant::ALL.iter().collect();
        assert_eq!(unique.len(), 13);
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
            (
                Variant::Yukon(YukonRules::Classic),
                r#"{"yukon":"classic"}"#,
            ),
            (
                Variant::Yukon(YukonRules::Russian),
                r#"{"yukon":"russian"}"#,
            ),
            (Variant::Yukon(YukonRules::Alaska), r#"{"yukon":"alaska"}"#),
            (
                Variant::Yukon(YukonRules::Moosehide),
                r#"{"yukon":"moosehide"}"#,
            ),
            (
                Variant::Yukon(YukonRules::Australian),
                r#"{"yukon":"australian"}"#,
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
    fn only_freecell_and_yukon_without_a_stock_have_no_stock() {
        for variant in Variant::ALL {
            let stock = deal(variant, 3).stock_status().is_some();
            assert_eq!(variant.has_stock(), stock, "{variant:?}");
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
