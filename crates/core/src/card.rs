//! Card identity: suit, color, rank and a unique id within a deal.

use serde::{Deserialize, Serialize};

/// One of the four suits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

/// The color of a suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    Red,
    Black,
}

impl Suit {
    /// All suits in declaration order.
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    /// Red for diamonds and hearts, black otherwise.
    pub fn color(self) -> Color {
        match self {
            Suit::Diamonds | Suit::Hearts => Color::Red,
            Suit::Clubs | Suit::Spades => Color::Black,
        }
    }
}

/// A card rank from ace to king. The model carries no numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    /// All ranks from ace to king.
    pub const ALL: [Rank; 13] = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ];

    /// Position in [`Rank::ALL`], `0` for ace to `12` for king.
    pub fn index(self) -> u8 {
        self as u8
    }

    /// The next lower rank, `None` for an ace.
    pub fn lower(self) -> Option<Rank> {
        usize::from(self.index())
            .checked_sub(1)
            .and_then(|i| Rank::ALL.get(i).copied())
    }

    /// The next higher rank, `None` for a king.
    pub fn higher(self) -> Option<Rank> {
        Rank::ALL.get(usize::from(self.index()) + 1).copied()
    }
}

/// Identifier of a card, unique within one deal (Spider deals contain duplicate faces).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CardId(pub u16);

/// A physical card: identity, suit and rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub id: CardId,
    pub suit: Suit,
    pub rank: Rank,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors() {
        assert_eq!(Suit::Hearts.color(), Color::Red);
        assert_eq!(Suit::Diamonds.color(), Color::Red);
        assert_eq!(Suit::Clubs.color(), Color::Black);
        assert_eq!(Suit::Spades.color(), Color::Black);
    }

    #[test]
    fn rank_indices_follow_all() {
        for (i, rank) in Rank::ALL.iter().enumerate() {
            assert_eq!(usize::from(rank.index()), i);
        }
    }

    #[test]
    fn lower_and_higher_stop_at_the_ends() {
        assert_eq!(Rank::Ace.lower(), None);
        assert_eq!(Rank::King.higher(), None);
        assert_eq!(Rank::Ace.higher(), Some(Rank::Two));
        assert_eq!(Rank::King.lower(), Some(Rank::Queen));
        assert_eq!(Rank::Seven.lower(), Some(Rank::Six));
        assert_eq!(Rank::Seven.higher(), Some(Rank::Eight));
    }

    #[test]
    fn rank_order_matches_index() {
        assert!(Rank::Ace < Rank::Two);
        assert!(Rank::Queen < Rank::King);
    }
}
