//! Shared helpers for the Spider tests. Cards are written as `"AS"`, `"10H"`, `"KD"`.

mod auto;
mod deal;
mod hints;
mod rules;

use super::{Column, Spider};
use crate::card::{Card, CardId, Rank, Suit};
use crate::game::Game;
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;

const RANKS: [&str; 13] = [
    "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
];
const SUITS: &str = "CDHS";
const SPADES_RUN: &str = "KS QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2S AS";
const HEARTS_RUN: &str = "KH QH JH 10H 9H 8H 7H 6H 5H 4H 3H 2H AH";

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

fn column(down: &str, up: &str) -> Column {
    Column {
        down: cards(down),
        up: cards(up),
    }
}

fn table() -> Spider {
    Spider::empty()
}

fn transfer(from: PileRef, to: PileRef, count: u8) -> Move {
    Move::Transfer { from, to, count }
}

fn tab(index: u8) -> PileRef {
    PileRef::Tableau(index)
}

fn found(index: u8) -> PileRef {
    PileRef::Foundation(index)
}

fn fill_empty_columns(game: &mut Spider) {
    for column in game.columns.iter_mut().filter(|c| c.up.is_empty()) {
        column.up = cards("KC");
    }
}

fn assert_rejected(game: &Spider, mv: Move, error: MoveError) {
    let mut copy = game.clone();
    assert_eq!(copy.apply(mv), Err(error), "{mv:?}");
    assert_eq!(copy.piles(), game.piles(), "{mv:?} changed the state");
    assert!(!game.legal_moves().contains(&mv), "{mv:?} listed as legal");
}

fn assert_accepted(game: &mut Spider, mv: Move) {
    assert!(
        game.legal_moves().contains(&mv),
        "{mv:?} not listed as legal"
    );
    assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
}

fn cards_of(game: &Spider, pile: PileRef) -> Vec<Card> {
    let snapshot = game.piles().into_iter().find(|p| p.pile == pile).unwrap();
    snapshot.cards.iter().map(|c| c.card).collect()
}
