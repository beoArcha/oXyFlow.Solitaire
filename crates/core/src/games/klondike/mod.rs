//! Klondike: seven columns, a stock drawn to the waste one or three cards at a time, four
//! foundations. The module follows the shared layout of the games: state and deal here, validation
//! and execution in `rules`, move generation and the hint in `moves`.

mod finish;
mod moves;
mod rules;
#[cfg(test)]
mod tests;

use crate::card::Card;
use crate::deck;
use crate::game::{DrawCount, Game, GameOptions, StockPasses, StockStatus};
use crate::games::progress::{self, HintProfile};
use crate::games::{index_ref, snapshot};
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

const COLUMNS: usize = 7;
const FOUNDATIONS: usize = 4;
const DECK_SIZE: usize = 52;

/// One tableau column. `up` is always a descending, alternating run; `up` is empty only when
/// `down` is empty too, because a face-down card is turned over as soon as it is exposed.
#[derive(Debug, Clone, Default)]
struct Column {
    down: Vec<Card>,
    up: Vec<Card>,
}

/// A Klondike game. The top card of every pile is the last element of its vector. `recycles`
/// counts the waste turned back into the stock.
#[derive(Debug, Clone)]
pub(crate) struct Klondike {
    draw: DrawCount,
    passes: StockPasses,
    any_empty: bool,
    stock: Vec<Card>,
    waste: Vec<Card>,
    foundations: [Vec<Card>; FOUNDATIONS],
    columns: [Column; COLUMNS],
    recycles: u32,
}

impl Klondike {
    #[cfg(test)]
    fn empty(draw: DrawCount) -> Self {
        Self::empty_with(draw, GameOptions::DEFAULT)
    }

    fn empty_with(draw: DrawCount, options: GameOptions) -> Self {
        Self {
            draw,
            passes: options.klondike_passes,
            any_empty: options.klondike_any_empty,
            stock: Vec::new(),
            waste: Vec::new(),
            foundations: Default::default(),
            columns: Default::default(),
            recycles: 0,
        }
    }

    /// Deals a shuffled deck from `seed` with default options.
    #[cfg(test)]
    pub(crate) fn deal(draw: DrawCount, seed: u64) -> Self {
        Self::deal_with(draw, seed, GameOptions::DEFAULT)
    }

    /// Deals a shuffled deck from `seed`: column *i* gets *i*+1 cards with the last one face up,
    /// the remaining 24 form the stock.
    pub(crate) fn deal_with(draw: DrawCount, seed: u64, options: GameOptions) -> Self {
        let mut cards = deck::standard();
        deck::shuffle_with_seed(&mut cards, seed);
        let mut game = Self::empty_with(draw, options);
        let mut rest = cards.into_iter();
        for (index, column) in game.columns.iter_mut().enumerate() {
            column.down = rest.by_ref().take(index + 1).collect();
            column.up.extend(column.down.pop());
        }
        game.stock = rest.collect();
        game
    }

    pub(super) fn recycles_left(&self) -> Option<u32> {
        match self.passes {
            StockPasses::Unlimited => None,
            StockPasses::Three => Some(2_u32.saturating_sub(self.recycles)),
            StockPasses::One => Some(0),
        }
    }

    pub(super) fn recycle_allowed(&self) -> bool {
        self.recycles_left().is_none_or(|left| left > 0)
    }
}

/// The hint profile of this game, chosen by the hints-only measurement of T-48.
const HINT_PROFILE: HintProfile = HintProfile::Lookahead;

impl Game for Klondike {
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
        progress::hint(self, HINT_PROFILE, progress::PLAY_KNOWLEDGE, || {
            self.suggest()
        })
    }

    fn is_won(&self) -> bool {
        self.foundations.iter().map(Vec::len).sum::<usize>() == DECK_SIZE
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        let foundations = self.foundations.iter().enumerate();
        let columns = self.columns.iter().enumerate();
        let mut piles = vec![
            snapshot(PileRef::Stock, &self.stock, &[]),
            snapshot(PileRef::Waste, &[], &self.waste),
        ];
        piles.extend(
            foundations.map(|(i, cards)| snapshot(PileRef::Foundation(index_ref(i)), &[], cards)),
        );
        piles
            .extend(columns.map(|(i, c)| snapshot(PileRef::Tableau(index_ref(i)), &c.down, &c.up)));
        piles
    }

    fn stock_status(&self) -> Option<StockStatus> {
        Some(StockStatus {
            recycles: self.recycles,
            recycles_left: self.recycles_left(),
            deals_left: None,
        })
    }

    fn forced_finish(&self) -> Option<Vec<Move>> {
        self.finishing_moves()
    }
}
