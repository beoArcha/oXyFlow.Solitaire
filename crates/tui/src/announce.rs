//! Plain-text names of the things on the board, built from localized card and pile names: piles,
//! cards, what a position shows and how many targets a pick has. The `Now:` row of the controls
//! strip is made of them; no color and no symbols, so the text reads the same to a screen reader.

use oxysol_core::{Card, PileRef, Rank, Suit};

use crate::i18n::{t, tf};

/// What the focused position shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Empty,
    FaceDown,
    Up(Card),
}

/// The localized name of a pile, numbered from one: `Tableau 3`, `Foundation 1`.
pub fn pile_name(pile: PileRef) -> String {
    match pile {
        PileRef::Stock => t("board.name.stock").into_owned(),
        PileRef::Waste => t("board.name.waste").into_owned(),
        PileRef::Foundation(index) => tf("board.name.foundation", &[("number", &ordinal(index))]),
        PileRef::Tableau(index) => tf("board.name.tableau", &[("number", &ordinal(index))]),
        PileRef::Cell(index) => tf("board.name.free_cell", &[("number", &ordinal(index))]),
        PileRef::Pyramid { row, index } => tf(
            "board.name.pyramid",
            &[("row", &ordinal(row)), ("position", &ordinal(index))],
        ),
    }
}

fn ordinal(index: u8) -> String {
    (u16::from(index) + 1).to_string()
}

/// The localized name of a card in words, for example `seven of hearts`.
pub fn card_name(card: Card) -> String {
    tf(
        "board.name.card",
        &[
            ("rank", &t(rank_key(card.rank))),
            ("suit", &t(suit_key(card.suit))),
        ],
    )
}

const fn rank_key(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "board.word.ace",
        Rank::Two => "board.word.two",
        Rank::Three => "board.word.three",
        Rank::Four => "board.word.four",
        Rank::Five => "board.word.five",
        Rank::Six => "board.word.six",
        Rank::Seven => "board.word.seven",
        Rank::Eight => "board.word.eight",
        Rank::Nine => "board.word.nine",
        Rank::Ten => "board.word.ten",
        Rank::Jack => "board.word.jack",
        Rank::Queen => "board.word.queen",
        Rank::King => "board.word.king",
    }
}

const fn suit_key(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "board.word.clubs",
        Suit::Diamonds => "board.word.diamonds",
        Suit::Hearts => "board.word.hearts",
        Suit::Spades => "board.word.spades",
    }
}

/// The localized words for what a position shows: `empty`, `face-down card`, or the card.
pub fn look_text(look: Look) -> String {
    match look {
        Look::Empty => t("board.name.empty").into_owned(),
        Look::FaceDown => t("board.name.face_down").into_owned(),
        Look::Up(card) => card_name(card),
    }
}

/// The localized count of targets: `1 target`, `3 targets`.
pub fn targets_text(targets: usize) -> String {
    if targets == 1 {
        return t("board.announce.targets_one").into_owned();
    }
    tf(
        "board.announce.targets_many",
        &[("count", &targets.to_string())],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn piles_are_numbered_from_one() {
        assert_eq!(pile_name(PileRef::Tableau(2)), "Tableau 3");
        assert_eq!(pile_name(PileRef::Foundation(0)), "Foundation 1");
        assert_eq!(pile_name(PileRef::Cell(3)), "Free cell 4");
        assert_eq!(pile_name(PileRef::Stock), "Stock");
        assert_eq!(
            pile_name(PileRef::Pyramid { row: 6, index: 0 }),
            "Row 7, card 1"
        );
    }
}
