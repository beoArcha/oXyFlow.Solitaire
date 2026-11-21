//! The Yukon position of the solver: columns with their face-down counts, the one-pass talon of
//! Australian Patience (stock and waste, order known), the foundation heights and where the engine
//! keeps each foundation. Which card fits onto which comes from the engine's building rule.

use crate::card::{Card, CardId, Rank, Suit};
use crate::game::YukonRules;
use crate::games::yukon::BuildRule;
use crate::pile::{PileRef, PileSnapshot};
use crate::solver::face::{self, Face, KING, NONE, RANKS, fill, hidden_count, rank, suit};
use crate::solver::hash::{PositionHash, column_hash, feed_unordered};

pub(super) const COLUMNS: usize = 7;
pub(super) const SUITS: usize = 4;
const DECK: u8 = 52;
const COLUMN_CAPACITY: usize = 52;
const TALON_CAPACITY: usize = 24;

fn card_of(face: Face) -> Option<Card> {
    Some(Card {
        id: CardId(0),
        suit: *Suit::ALL.get(usize::from(suit(face)))?,
        rank: *Rank::ALL.get(usize::from(rank(face)))?,
    })
}

/// The stock (top last) and the waste (top last). The stock is drawn once, one card at a time.
#[derive(Clone)]
pub(super) struct Talon {
    stock: [Face; TALON_CAPACITY],
    pub(super) stock_len: usize,
    waste: [Face; TALON_CAPACITY],
    waste_len: usize,
}

impl Talon {
    fn empty() -> Self {
        Self {
            stock: [NONE; TALON_CAPACITY],
            stock_len: 0,
            waste: [NONE; TALON_CAPACITY],
            waste_len: 0,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.stock_len == 0 && self.waste_len == 0
    }

    pub(super) fn waste_top(&self) -> Option<Face> {
        self.waste_len
            .checked_sub(1)
            .and_then(|top| self.waste.get(top).copied())
    }

    /// The waste top after `draws` more stock moves, when the stock holds that many cards.
    pub(super) fn top_after(&self, draws: usize) -> Option<Face> {
        if draws == 0 {
            return self.waste_top();
        }
        self.stock_len
            .checked_sub(draws)
            .and_then(|at| self.stock.get(at).copied())
    }

    pub(super) fn pop_waste(&mut self) {
        self.waste_len = self.waste_len.saturating_sub(1);
    }

    /// One `Move::Stock`: the top stock card to the waste. There is no redeal.
    pub(super) fn draw(&mut self) {
        let Some(top) = self.stock_len.checked_sub(1) else {
            return;
        };
        self.waste[self.waste_len] = self.stock[top];
        self.waste_len += 1;
        self.stock_len = top;
    }

    fn feed(&self, hash: &mut PositionHash) {
        hash.feed(u64::try_from(self.stock_len).unwrap_or(u64::MAX));
        hash.feed_bytes(&self.waste[..self.waste_len]);
    }
}

#[derive(Clone)]
pub(in crate::solver) struct Position {
    build: BuildRule,
    pub(super) cols: [[Face; COLUMN_CAPACITY]; COLUMNS],
    pub(super) len: [usize; COLUMNS],
    pub(super) hidden: [usize; COLUMNS],
    pub(super) talon: Talon,
    pub(super) found: [u8; SUITS],
    slot: [u8; SUITS],
}

impl Position {
    /// Reads the snapshot of a game of `rules`. `None` when it does not look like one.
    pub(in crate::solver) fn from_piles(rules: YukonRules, piles: &[PileSnapshot]) -> Option<Self> {
        let mut position = Self {
            build: rules.build(),
            cols: [[NONE; COLUMN_CAPACITY]; COLUMNS],
            len: [0; COLUMNS],
            hidden: [0; COLUMNS],
            talon: Talon::empty(),
            found: [0; SUITS],
            slot: [NONE; SUITS],
        };
        let stock = face::cards_of(piles, PileRef::Stock);
        position.talon.stock_len = usize::from(fill(&mut position.talon.stock, stock)?);
        let waste = face::cards_of(piles, PileRef::Waste);
        position.talon.waste_len = usize::from(fill(&mut position.talon.waste, waste)?);
        for i in 0..COLUMNS {
            let cards = face::cards_of(piles, PileRef::Tableau(u8::try_from(i).ok()?));
            position.len[i] = usize::from(fill(&mut position.cols[i], cards)?);
            position.hidden[i] = usize::from(hidden_count(cards)?);
        }
        for i in 0..SUITS {
            let cards = face::cards_of(piles, PileRef::Foundation(u8::try_from(i).ok()?));
            let Some(last) = cards.last() else {
                continue;
            };
            let suit = usize::from(suit(face::face(last.card)));
            position.found[suit] = u8::try_from(cards.len()).ok()?;
            position.slot[suit] = u8::try_from(i).ok()?;
        }
        Some(position)
    }

    pub(super) fn top(&self, column: usize) -> Option<Face> {
        self.len[column]
            .checked_sub(1)
            .map(|top| self.cols[column][top])
    }

    pub(super) fn face_up(&self, column: usize) -> usize {
        self.len[column] - self.hidden[column]
    }

    /// Whether `card` may be built onto `top` under the rule set's building rule.
    pub(super) fn fits(&self, top: Face, card: Face) -> bool {
        card_of(top)
            .zip(card_of(card))
            .is_some_and(|(top, card)| self.build.fits(top, card))
    }

    /// Whether a group with `card` at its bottom may go onto `column`: onto a top it fits, or a
    /// king onto an empty column.
    pub(super) fn accepts(&self, column: usize, card: Face) -> bool {
        match self.top(column) {
            Some(top) => self.fits(top, card),
            None => rank(card) == KING,
        }
    }

    pub(super) fn is_home(&self, card: Face) -> bool {
        self.found[usize::from(suit(card))] > rank(card)
    }

    /// A card may go to its foundation for good when every card that could still be built onto
    /// it is home already (the T-29 rule of Klondike, read through the building rule).
    pub(super) fn is_safe(&self, card: Face) -> bool {
        let neighbours = [rank(card).checked_sub(1), Some(rank(card) + 1)];
        neighbours
            .into_iter()
            .flatten()
            .filter(|&near| near < RANKS)
            .flat_map(|near| (0..4u8).map(move |s| s * RANKS + near))
            .all(|other| self.is_home(other) || !self.fits(card, other))
    }

    pub(super) fn foundation_accepts(&self, card: Face) -> bool {
        self.found[usize::from(suit(card))] == rank(card)
    }

    /// The foundation card of `suit` on top, if the foundation has one.
    pub(super) fn foundation_top(&self, suit: usize) -> Option<Face> {
        let height = self.found[suit];
        let suit = u8::try_from(suit).ok()?;
        height.checked_sub(1).map(|top| suit * RANKS + top)
    }

    /// The engine foundation that takes `card`: its own one, or the first empty one for an ace.
    pub(super) fn foundation_index(&self, card: Face) -> u8 {
        let own = usize::from(suit(card));
        if self.found[own] > 0 {
            return self.slot[own];
        }
        (0..SUITS)
            .filter_map(|i| u8::try_from(i).ok())
            .find(|&i| (0..SUITS).all(|s| self.found[s] == 0 || self.slot[s] != i))
            .unwrap_or(0)
    }

    pub(super) fn foundation_slot(&self, suit: usize) -> u8 {
        self.slot[suit]
    }

    pub(super) fn place_home(&mut self, card: Face) {
        let own = usize::from(suit(card));
        if self.found[own] == 0 {
            self.slot[own] = self.foundation_index(card);
        }
        self.found[own] += 1;
    }

    pub(super) fn lift_home(&mut self, suit: usize) {
        self.found[suit] = self.found[suit].saturating_sub(1);
    }

    /// Removes the top card of `column` and turns over the card it uncovers.
    pub(super) fn pop(&mut self, column: usize) {
        self.len[column] -= 1;
        self.flip(column);
    }

    pub(super) fn push(&mut self, column: usize, card: Face) {
        self.cols[column][self.len[column]] = card;
        self.len[column] += 1;
    }

    fn flip(&mut self, column: usize) {
        if self.len[column] == self.hidden[column] && self.hidden[column] > 0 {
            self.hidden[column] -= 1;
        }
    }

    pub(super) fn move_group(&mut self, from: usize, to: usize, count: usize) {
        let start = self.len[from] - count;
        for offset in 0..count {
            let card = self.cols[from][start + offset];
            self.push(to, card);
        }
        self.len[from] = start;
        self.flip(from);
    }

    /// Whether the cards of `column` never rise in rank from the bottom up, so the lowest card
    /// left anywhere is always on top of its column.
    pub(super) fn is_descending(&self, column: usize) -> bool {
        self.cols[column][..self.len[column]]
            .windows(2)
            .all(|pair| rank(pair[0]) >= rank(pair[1]))
    }

    pub(super) fn is_won(&self) -> bool {
        self.found.iter().copied().sum::<u8>() == DECK
    }

    pub(super) fn key(&self) -> u128 {
        let columns = (0..COLUMNS)
            .map(|i| {
                column_hash(
                    u8::try_from(self.hidden[i]).unwrap_or(0),
                    &self.cols[i][..self.len[i]],
                )
            })
            .collect();
        let mut hash = PositionHash::new();
        feed_unordered(&mut hash, columns);
        hash.feed_bytes(&self.found);
        self.talon.feed(&mut hash);
        hash.key()
    }
}
