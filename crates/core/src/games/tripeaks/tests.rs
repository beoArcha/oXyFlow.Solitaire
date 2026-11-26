//! Shared helpers for the TriPeaks tests. Cards are written as `"AS"`, `"10H"`, `"KD"`.

mod deal;
mod finish;
mod hints;
mod rules;

use super::{ROW_LENS, ROWS, TriPeaks};
use crate::card::{Card, CardId, Rank, Suit};
use crate::game::Game;
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, TableCard};

const RANKS: [&str; 13] = [
    "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
];
const SUITS: &str = "CDHS";

fn card(code: &str) -> Card {
    let (rank, suit) = code.split_at(code.len() - 1);
    let rank_index = RANKS.iter().position(|r| *r == rank).unwrap();
    let suit_index = SUITS.find(suit).unwrap();
    Card {
        id: CardId(u16::try_from(suit_index * 13 + rank_index).unwrap()),
        suit: Suit::ALL[suit_index],
        rank: Rank::ALL[rank_index],
    }
}

fn cards(codes: &str) -> Vec<Card> {
    codes.split_whitespace().map(card).collect()
}

fn empty_game(waste: &str, stock: &str) -> TriPeaks {
    TriPeaks {
        rows: std::array::from_fn(|row| vec![None; ROW_LENS[row]]),
        stock: cards(stock),
        waste: cards(waste),
    }
}

fn put(game: &mut TriPeaks, row: usize, index: usize, code: &str, face_up: bool) {
    assert!(row < ROWS && index < ROW_LENS[row]);
    game.rows[row][index] = Some(TableCard {
        card: card(code),
        face_up,
    });
}

fn peak(row: u8, index: u8) -> PileRef {
    PileRef::Pyramid { row, index }
}

fn play(row: u8, index: u8) -> Move {
    Move::Transfer {
        from: peak(row, index),
        to: PileRef::Waste,
        count: 1,
    }
}

fn assert_rejected(game: &TriPeaks, mv: Move, error: MoveError) {
    let mut copy = game.clone();
    assert_eq!(copy.apply(mv), Err(error), "{mv:?}");
    assert_eq!(copy.piles(), game.piles(), "{mv:?} changed the state");
    assert!(!game.legal_moves().contains(&mv), "{mv:?} listed as legal");
}

fn assert_accepted(game: &mut TriPeaks, mv: Move) {
    assert!(
        game.legal_moves().contains(&mv),
        "{mv:?} not listed as legal"
    );
    assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
}

fn cards_of(game: &TriPeaks, pile: PileRef) -> Vec<Card> {
    let snapshot = game.piles().into_iter().find(|p| p.pile == pile).unwrap();
    snapshot.cards.iter().map(|c| c.card).collect()
}
