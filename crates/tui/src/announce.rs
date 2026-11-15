//! One-line plain-text descriptions of the board for the status line: where the focus is and what
//! the last event was, built from localized card and pile names. No color and no symbols, so the
//! text reads the same to a screen reader.

use oxysol_core::{Card, PileRef, Rank, Suit};

use crate::i18n::{t, tf};

/// What the focused position shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Empty,
    FaceDown,
    Up(Card),
}

/// Cards picked up and waiting for a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    pub source: PileRef,
    pub count: u8,
    pub targets: usize,
}

/// The focus of the board: the pile under the cursor, what it shows, and the cards held, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus {
    pub pile: PileRef,
    pub look: Look,
    pub held: Option<Held>,
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

fn selection_text(look: Look, count: u8) -> String {
    if count > 1 {
        return tf(
            "board.announce.selected_many",
            &[("count", &count.to_string())],
        );
    }
    tf("board.announce.selected", &[("what", &look_text(look))])
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

/// Describes the focus, for example `Tableau 3: seven of hearts selected, 2 targets`.
pub fn focus(focus: &Focus) -> String {
    let what = match focus.held {
        Some(held) if held.source == focus.pile => selection_text(focus.look, held.count),
        _ => look_text(focus.look),
    };
    let base = tf(
        "board.announce.focus",
        &[("pile", &pile_name(focus.pile)), ("what", &what)],
    );
    match focus.held {
        Some(held) => tf(
            "board.announce.with_targets",
            &[("text", &base), ("targets", &targets_text(held.targets))],
        ),
        None => base,
    }
}

/// Joins a description of the focus with the last event, skipping an empty part.
pub fn announce(focus: &str, last_event: &str) -> String {
    match (focus.is_empty(), last_event.is_empty()) {
        (true, true) => String::new(),
        (false, true) => focus.to_owned(),
        (true, false) => last_event.to_owned(),
        (false, false) => tf(
            "board.announce.line",
            &[("focus", focus), ("event", last_event)],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxysol_core::CardId;

    fn card(suit: Suit, rank: Rank) -> Card {
        Card {
            id: CardId(0),
            suit,
            rank,
        }
    }

    #[test]
    fn empty_parts_are_skipped() {
        assert_eq!(announce("", ""), "");
        assert_eq!(announce("Stock: empty", ""), "Stock: empty");
        assert_eq!(announce("", "Cards moved."), "Cards moved.");
        assert_eq!(
            announce("Stock: empty", "Cards moved."),
            "Stock: empty. Cards moved."
        );
    }

    #[test]
    fn piles_are_numbered_from_one() {
        assert_eq!(pile_name(PileRef::Tableau(2)), "Tableau 3");
        assert_eq!(pile_name(PileRef::Foundation(0)), "Foundation 1");
        assert_eq!(pile_name(PileRef::Cell(3)), "Free cell 4");
        assert_eq!(pile_name(PileRef::Stock), "Stock");
        assert_eq!(
            pile_name(PileRef::Pyramid { row: 6, index: 0 }),
            "Pyramid row 7, position 1"
        );
    }

    #[test]
    fn focus_on_a_selected_card_with_targets() {
        let focus = Focus {
            pile: PileRef::Tableau(2),
            look: Look::Up(card(Suit::Hearts, Rank::Seven)),
            held: Some(Held {
                source: PileRef::Tableau(2),
                count: 1,
                targets: 2,
            }),
        };
        assert_eq!(
            super::focus(&focus),
            "Tableau 3: seven of hearts selected, 2 targets"
        );
    }

    #[test]
    fn focus_on_plain_cards_and_empty_piles() {
        let face_down = Focus {
            pile: PileRef::Tableau(0),
            look: Look::FaceDown,
            held: None,
        };
        assert_eq!(super::focus(&face_down), "Tableau 1: face-down card");
        let empty = Focus {
            pile: PileRef::Waste,
            look: Look::Empty,
            held: None,
        };
        assert_eq!(super::focus(&empty), "Waste: empty");
    }

    #[test]
    fn a_run_counts_its_cards_and_one_target_is_singular() {
        let focus = Focus {
            pile: PileRef::Tableau(4),
            look: Look::Up(card(Suit::Spades, Rank::King)),
            held: Some(Held {
                source: PileRef::Tableau(4),
                count: 3,
                targets: 1,
            }),
        };
        assert_eq!(
            super::focus(&focus),
            "Tableau 5: 3 cards selected, 1 target"
        );
    }

    #[test]
    fn a_held_run_elsewhere_names_only_the_targets() {
        let focus = Focus {
            pile: PileRef::Foundation(1),
            look: Look::Up(card(Suit::Clubs, Rank::Two)),
            held: Some(Held {
                source: PileRef::Tableau(0),
                count: 1,
                targets: 3,
            }),
        };
        assert_eq!(
            super::focus(&focus),
            "Foundation 2: two of clubs, 3 targets"
        );
    }
}
