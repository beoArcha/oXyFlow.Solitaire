//! Suit pictograms as tables of rows, in three scales and two symbol sets. The Unicode rows use
//! only WGL4 block characters, the ASCII rows only printable ASCII; the four shapes differ in both
//! sets, so the suit reads without color.

use oxysol_core::Suit;

/// Size class of the card centre, chosen from the interior of the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Scale {
    Large,
    Medium,
    Small,
}

pub(super) type Rows = &'static [&'static str];

impl Scale {
    pub(super) const fn suit_height(self) -> u16 {
        match self {
            Self::Large => 4,
            Self::Medium => 3,
            Self::Small => 2,
        }
    }

    pub(super) const fn suit_width(self) -> u16 {
        match self {
            Self::Large => 7,
            Self::Medium => 5,
            Self::Small => 5,
        }
    }
}

const LARGE_CLUBS: Rows = &[" ▄███▄ ", "▄▄█▀█▄▄", "███▀███", "  ▄█▄  "];
const LARGE_DIAMONDS: Rows = &["  ▄█▄  ", "▄█████▄", "▀█████▀", "  ▀█▀  "];
const LARGE_HEARTS: Rows = &["▄██▄██▄", "███████", " ▀███▀ ", "   ▀   "];
const LARGE_SPADES: Rows = &["  ▄█▄  ", "▄█████▄", "▀██▀██▀", "  ▄█▄  "];

const MEDIUM_CLUBS: Rows = &[" ▄█▄ ", "▄█▀█▄", "  █  "];
const MEDIUM_DIAMONDS: Rows = &[" ▄█▄ ", "█████", " ▀█▀ "];
const MEDIUM_HEARTS: Rows = &["██▄██", "▀███▀", "  ▀  "];
const MEDIUM_SPADES: Rows = &[" ▄█▄ ", "█████", "  █  "];

const SMALL_CLUBS: Rows = &[" ▄█▄ ", "▄█▀█▄"];
const SMALL_DIAMONDS: Rows = &[" ▄█▄ ", " ▀█▀ "];
const SMALL_HEARTS: Rows = &["██▄██", " ▀█▀ "];
const SMALL_SPADES: Rows = &[" ▄█▄ ", "▀▀█▀▀"];

const ASCII_LARGE_CLUBS: Rows = &["  ###  ", " ##### ", "### ###", "  /#\\  "];
const ASCII_LARGE_DIAMONDS: Rows = &["   ^   ", " /###\\ ", " \\###/ ", "   v   "];
const ASCII_LARGE_HEARTS: Rows = &[" ## ## ", "#######", " \\###/ ", "   v   "];
const ASCII_LARGE_SPADES: Rows = &["   ^   ", " /###\\ ", "#######", "  /#\\  "];

const ASCII_MEDIUM_CLUBS: Rows = &[" ### ", "## ##", " /#\\ "];
const ASCII_MEDIUM_DIAMONDS: Rows = &["  ^  ", "<###>", "  v  "];
const ASCII_MEDIUM_HEARTS: Rows = &["## ##", "#####", " \\#/ "];
const ASCII_MEDIUM_SPADES: Rows = &["  ^  ", "#####", " /#\\ "];

const ASCII_SMALL_CLUBS: Rows = &[" ### ", "#/#\\#"];
const ASCII_SMALL_DIAMONDS: Rows = &[" /#\\ ", " \\#/ "];
const ASCII_SMALL_HEARTS: Rows = &["## ##", " \\#/ "];
const ASCII_SMALL_SPADES: Rows = &[" /#\\ ", "#####"];

/// The rows of a suit pictogram.
pub(super) fn suit_art(scale: Scale, suit: Suit, unicode: bool) -> Rows {
    match (unicode, scale, suit) {
        (true, Scale::Large, Suit::Clubs) => LARGE_CLUBS,
        (true, Scale::Large, Suit::Diamonds) => LARGE_DIAMONDS,
        (true, Scale::Large, Suit::Hearts) => LARGE_HEARTS,
        (true, Scale::Large, Suit::Spades) => LARGE_SPADES,
        (true, Scale::Medium, Suit::Clubs) => MEDIUM_CLUBS,
        (true, Scale::Medium, Suit::Diamonds) => MEDIUM_DIAMONDS,
        (true, Scale::Medium, Suit::Hearts) => MEDIUM_HEARTS,
        (true, Scale::Medium, Suit::Spades) => MEDIUM_SPADES,
        (true, Scale::Small, Suit::Clubs) => SMALL_CLUBS,
        (true, Scale::Small, Suit::Diamonds) => SMALL_DIAMONDS,
        (true, Scale::Small, Suit::Hearts) => SMALL_HEARTS,
        (true, Scale::Small, Suit::Spades) => SMALL_SPADES,
        (false, Scale::Large, Suit::Clubs) => ASCII_LARGE_CLUBS,
        (false, Scale::Large, Suit::Diamonds) => ASCII_LARGE_DIAMONDS,
        (false, Scale::Large, Suit::Hearts) => ASCII_LARGE_HEARTS,
        (false, Scale::Large, Suit::Spades) => ASCII_LARGE_SPADES,
        (false, Scale::Medium, Suit::Clubs) => ASCII_MEDIUM_CLUBS,
        (false, Scale::Medium, Suit::Diamonds) => ASCII_MEDIUM_DIAMONDS,
        (false, Scale::Medium, Suit::Hearts) => ASCII_MEDIUM_HEARTS,
        (false, Scale::Medium, Suit::Spades) => ASCII_MEDIUM_SPADES,
        (false, Scale::Small, Suit::Clubs) => ASCII_SMALL_CLUBS,
        (false, Scale::Small, Suit::Diamonds) => ASCII_SMALL_DIAMONDS,
        (false, Scale::Small, Suit::Hearts) => ASCII_SMALL_HEARTS,
        (false, Scale::Small, Suit::Spades) => ASCII_SMALL_SPADES,
    }
}
