//! Shared helpers for the Golf tests. Cards are written as `"AS"`, `"10H"`, `"KD"`.

mod deal;
mod finish;
mod hints;
mod rules;

use super::{COLUMNS, Golf};
use crate::card::{Card, CardId, Rank, Suit};
use crate::game::Game;
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;

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

/// A table with the given waste (bottom to top) and stock (top last) and no column cards.
fn table(waste: &str, stock: &str) -> Golf {
    Golf {
        wrap: false,
        king_stops: false,
        stock: cards(stock),
        waste: cards(waste),
        columns: Default::default(),
    }
}

fn set_column(game: &mut Golf, index: usize, codes: &str) {
    game.columns[index] = cards(codes);
}

fn tab(index: u8) -> PileRef {
    PileRef::Tableau(index)
}

fn play(index: u8) -> Move {
    Move::Transfer {
        from: tab(index),
        to: PileRef::Waste,
        count: 1,
    }
}

fn assert_rejected(game: &Golf, mv: Move, error: MoveError) {
    let mut copy = game.clone();
    assert_eq!(copy.apply(mv), Err(error), "{mv:?}");
    assert_eq!(copy.piles(), game.piles(), "{mv:?} changed the state");
    assert!(!game.legal_moves().contains(&mv), "{mv:?} listed as legal");
}

fn assert_accepted(game: &mut Golf, mv: Move) {
    assert!(
        game.legal_moves().contains(&mv),
        "{mv:?} not listed as legal"
    );
    assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
}

fn cards_of(game: &Golf, pile: PileRef) -> Vec<Card> {
    let snapshot = game.piles().into_iter().find(|p| p.pile == pile).unwrap();
    snapshot.cards.iter().map(|c| c.card).collect()
}
