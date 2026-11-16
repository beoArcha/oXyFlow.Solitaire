//! Shared helpers for the Pyramid tests. Cards are written as `"AS"`, `"10H"`, `"KD"`; a removed
//! pyramid position is written as `"."`.

mod deal;
mod finish;
mod hints;
mod legal;
mod rules;
mod stock;

use super::{Pyramid, passes, value};
use crate::card::{Card, CardId, Rank, Suit};
use crate::game::{Game, PyramidRules};
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;

const RANKS: [&str; 13] = [
    "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
];
const SUITS: &str = "CDHS";
const REMOVED: &str = ".";

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

fn slots(codes: &str) -> Vec<Option<Card>> {
    codes
        .split_whitespace()
        .map(|code| (code != REMOVED).then(|| card(code)))
        .collect()
}

fn empty(rules: PyramidRules) -> Pyramid {
    Pyramid {
        rules,
        rows: std::array::from_fn(|row| vec![None; row + 1]),
        stock: Vec::new(),
        waste: Vec::new(),
        foundation: Vec::new(),
        recycles_left: passes(rules).saturating_sub(1),
    }
}

/// A game with the pyramid rows, the stock and the waste given as code lists. The stock and waste
/// tops are their last codes.
fn table(rules: PyramidRules, rows: [&str; 7], stock: &str, waste: &str) -> Pyramid {
    let mut game = empty(rules);
    for (row, codes) in game.rows.iter_mut().zip(rows) {
        *row = slots(codes);
    }
    game.stock = cards(stock);
    game.waste = cards(waste);
    game
}

fn pyr(row: u8, index: u8) -> PileRef {
    PileRef::Pyramid { row, index }
}

fn remove(first: PileRef, second: Option<PileRef>) -> Move {
    Move::Remove { first, second }
}

fn reversed(mv: Move) -> Move {
    match mv {
        Move::Remove {
            first,
            second: Some(second),
        } => Move::Remove {
            first: second,
            second: Some(first),
        },
        other => other,
    }
}

fn assert_rejected(game: &Pyramid, mv: Move, error: MoveError) {
    let mut copy = game.clone();
    assert_eq!(copy.apply(mv), Err(error), "{mv:?}");
    assert_eq!(copy.piles(), game.piles(), "{mv:?} changed the state");
    assert!(!game.legal_moves().contains(&mv), "{mv:?} listed as legal");
}

fn assert_accepted(game: &mut Pyramid, mv: Move) {
    assert!(
        game.legal_moves().contains(&mv),
        "{mv:?} not listed as legal"
    );
    assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
}

fn cards_of(game: &Pyramid, pile: PileRef) -> Vec<Card> {
    let snapshot = game.piles().into_iter().find(|p| p.pile == pile).unwrap();
    snapshot.cards.iter().map(|c| c.card).collect()
}

fn code(card: &Card) -> String {
    let suit_index = Suit::ALL.iter().position(|s| *s == card.suit).unwrap();
    let suit = SUITS.chars().nth(suit_index).unwrap();
    format!("{}{}", RANKS[usize::from(card.rank.index())], suit)
}
