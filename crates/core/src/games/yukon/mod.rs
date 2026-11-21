//! Yukon and its family: seven columns, four foundations; Australian Patience adds a one-pass stock. Any face-up card moves with
//! every card above it, in or out of sequence; only the bottom card of the group must fit.
//!
//! The module follows the shared layout of the games: state and deal here, validation and
//! execution in `rules`, move generation and the hint in `moves`, the forced win in `finish`.
//!
//! Extension points for the family (P4-10b…e): a rule set is a [`YukonRules`] value. Its
//! building rule is `setup::BuildRule` (returned by `YukonRules::build`) and its layout a
//! `setup::Deal` (returned by `YukonRules::deal_plan`). Russian Solitaire, Alaska and Moosehide
//! add a `YukonRules` value, a `BuildRule` variant with its arm in `BuildRule::fits` if the rule is
//! new, and the two `match` arms in `setup`; the move code stays as it is. A deal with
//! `Deal::stock` above 0 (Australian Patience) turns on the stock and the waste: [`Move::Stock`]
//! draws one card, the waste top moves like a column card, and there is no redeal.

mod finish;
mod moves;
mod rules;
mod setup;
#[cfg(test)]
mod tests;

pub(crate) use setup::BuildRule;

use crate::card::Card;
use crate::deck;
use crate::game::{Game, StockStatus, YukonRules};
use crate::games::{index_ref, snapshot};
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

const COLUMNS: usize = 7;
const FOUNDATIONS: usize = 4;
const DECK_SIZE: usize = 52;

/// One tableau column. `up` holds the face-up cards in any order; it is empty only when `down`
/// is empty too, because a face-down card is turned over as soon as it is exposed.
#[derive(Debug, Clone, Default)]
struct Column {
    down: Vec<Card>,
    up: Vec<Card>,
}

/// A game of the Yukon family. The top card of every pile is the last element of its vector.
#[derive(Debug, Clone)]
pub(crate) struct Yukon {
    rules: YukonRules,
    stock: Vec<Card>,
    waste: Vec<Card>,
    foundations: [Vec<Card>; FOUNDATIONS],
    columns: [Column; COLUMNS],
}

impl Yukon {
    fn empty(rules: YukonRules) -> Self {
        Self {
            rules,
            stock: Vec::new(),
            waste: Vec::new(),
            foundations: Default::default(),
            columns: Default::default(),
        }
    }

    /// Deals a shuffled deck from `seed` as the `setup::Deal` of `rules` describes.
    pub(crate) fn deal(rules: YukonRules, seed: u64) -> Self {
        let mut cards = deck::standard();
        deck::shuffle_with_seed(&mut cards, seed);
        let plan = rules.deal_plan();
        let mut game = Self::empty(rules);
        let mut rest = cards.into_iter();
        let counts = plan.down.into_iter().zip(plan.up);
        for (column, (down, up)) in game.columns.iter_mut().zip(counts) {
            column.down = rest.by_ref().take(down).collect();
            column.up = rest.by_ref().take(up).collect();
        }
        game.stock = rest.take(plan.stock).collect();
        game
    }
}

impl Game for Yukon {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        if self.is_won() {
            return Err(MoveError::Unsupported);
        }
        self.execute(mv)
    }

    fn legal_moves(&self) -> Vec<Move> {
        if self.is_won() {
            return Vec::new();
        }
        self.enumerate_moves()
    }

    fn hint(&self) -> Option<Move> {
        if self.is_won() {
            return None;
        }
        self.suggest()
    }

    fn is_won(&self) -> bool {
        self.foundations.iter().map(Vec::len).sum::<usize>() == DECK_SIZE
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        let foundations = self
            .foundations
            .iter()
            .enumerate()
            .map(|(i, cards)| snapshot(PileRef::Foundation(index_ref(i)), &[], cards));
        let columns = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| snapshot(PileRef::Tableau(index_ref(i)), &c.down, &c.up));
        let stock = self.rules.has_stock().then(|| {
            [
                snapshot(PileRef::Stock, &self.stock, &[]),
                snapshot(PileRef::Waste, &[], &self.waste),
            ]
        });
        stock
            .into_iter()
            .flatten()
            .chain(foundations)
            .chain(columns)
            .collect()
    }

    fn stock_status(&self) -> Option<StockStatus> {
        self.rules.has_stock().then_some(StockStatus {
            recycles: 0,
            recycles_left: Some(0),
            deals_left: None,
        })
    }

    fn forced_finish(&self) -> Option<Vec<Move>> {
        self.finishing_moves()
    }
}
