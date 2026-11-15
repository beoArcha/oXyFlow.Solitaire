//! Helpers shared by the flow tests: a fixed seed and a compact text form of a table.

use oxysol_core::{PileRef, PileSnapshot, Rank, Suit, TableCard};

pub const SEED: u64 = 42;

fn rank_code(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "A",
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "J",
        Rank::Queen => "Q",
        Rank::King => "K",
    }
}

fn suit_code(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "C",
        Suit::Diamonds => "D",
        Suit::Hearts => "H",
        Suit::Spades => "S",
    }
}

/// A card as `7H`, with a `*` after a face-down card.
fn card_code(table_card: &TableCard) -> String {
    let face = if table_card.face_up { "" } else { "*" };
    let card = table_card.card;
    format!("{}{}{face}", rank_code(card.rank), suit_code(card.suit))
}

fn pile_label(pile: PileRef) -> String {
    match pile {
        PileRef::Stock => "Stock".to_owned(),
        PileRef::Waste => "Waste".to_owned(),
        PileRef::Foundation(i) => format!("Foundation {i}"),
        PileRef::Tableau(i) => format!("Tableau {i}"),
        PileRef::Cell(i) => format!("Cell {i}"),
        PileRef::Pyramid { row, index } => format!("Pyramid {row}.{index}"),
    }
}

/// One line per pile in `piles()` order: `Tableau 2: 4S* 9D* KH`, bottom to top.
pub fn layout(piles: &[PileSnapshot]) -> String {
    let lines: Vec<String> = piles
        .iter()
        .map(|pile| {
            let cards: String = pile
                .cards
                .iter()
                .map(|card| format!(" {}", card_code(card)))
                .collect();
            format!("{}:{cards}", pile_label(pile.pile))
        })
        .collect();
    lines.join("\n")
}
