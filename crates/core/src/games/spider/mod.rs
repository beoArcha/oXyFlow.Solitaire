//! Spider: ten columns built down in any suit, a stock dealt one card onto every column, eight
//! foundations that collect complete same-suit runs. The module follows the shared layout of the
//! games: state and deal here, validation and execution in `rules`, move generation and the hint
//! in `moves`.

mod moves;
mod rules;
#[cfg(test)]
mod tests;

use crate::card::{Card, Suit};
use crate::deck;
use crate::game::{Game, SuitCount};
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot, TableCard};

const COLUMNS: usize = 10;
const FOUNDATIONS: usize = 8;
const DECK_SIZE: usize = 104;
const RUN_SIZE: usize = 13;
const TALL_COLUMNS: usize = 4;
const TALL_SIZE: usize = 6;
const SHORT_SIZE: usize = 5;

/// One tableau column. `up` is empty only when `down` is empty too, because a face-down card is
/// turned over as soon as it is exposed. Unlike Klondike, `up` is not necessarily a run: a stock
/// deal may put any card on top, so only its same-suit descending tail can move.
#[derive(Debug, Clone, Default)]
struct Column {
    down: Vec<Card>,
    up: Vec<Card>,
}

/// A Spider game. The top card of every pile is the last element of its vector. A foundation holds
/// one complete run, king first.
#[derive(Debug, Clone)]
pub(crate) struct Spider {
    stock: Vec<Card>,
    foundations: [Vec<Card>; FOUNDATIONS],
    columns: [Column; COLUMNS],
}

impl Spider {
    fn empty() -> Self {
        Self {
            stock: Vec::new(),
            foundations: Default::default(),
            columns: Default::default(),
        }
    }

    /// Deals a shuffled double deck of `suits` from `seed`: the first four columns get six cards,
    /// the other six get five, the last card of each is face up, the remaining 50 form the stock.
    pub(crate) fn deal(suits: SuitCount, seed: u64) -> Self {
        let mut cards = match suits {
            SuitCount::One => deck::build(&[Suit::Spades], 8),
            SuitCount::Two => deck::build(&[Suit::Spades, Suit::Hearts], 4),
            SuitCount::Four => deck::build(&Suit::ALL, 2),
        };
        deck::shuffle_with_seed(&mut cards, seed);
        let mut game = Self::empty();
        let mut rest = cards.into_iter();
        for (index, column) in game.columns.iter_mut().enumerate() {
            let size = if index < TALL_COLUMNS {
                TALL_SIZE
            } else {
                SHORT_SIZE
            };
            column.down = rest.by_ref().take(size).collect();
            column.up.extend(column.down.pop());
        }
        game.stock = rest.collect();
        game
    }
}

fn snapshot(pile: PileRef, down: &[Card], up: &[Card]) -> PileSnapshot {
    let face = |face_up| move |&card| TableCard { card, face_up };
    PileSnapshot {
        pile,
        cards: down
            .iter()
            .map(face(false))
            .chain(up.iter().map(face(true)))
            .collect(),
    }
}

fn index_ref(index: usize) -> u8 {
    u8::try_from(index).unwrap_or(u8::MAX)
}

impl Game for Spider {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        self.execute(mv)
    }

    fn legal_moves(&self) -> Vec<Move> {
        self.enumerate_moves()
    }

    fn hint(&self) -> Option<Move> {
        self.suggest()
    }

    fn is_won(&self) -> bool {
        self.foundations.iter().map(Vec::len).sum::<usize>() == DECK_SIZE
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        let foundations = self.foundations.iter().enumerate();
        let columns = self.columns.iter().enumerate();
        let mut piles = vec![snapshot(PileRef::Stock, &self.stock, &[])];
        piles.extend(
            foundations.map(|(i, cards)| snapshot(PileRef::Foundation(index_ref(i)), &[], cards)),
        );
        piles
            .extend(columns.map(|(i, c)| snapshot(PileRef::Tableau(index_ref(i)), &c.down, &c.up)));
        piles
    }
}
