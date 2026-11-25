//! Prints what the cards and the boards look like. Run with
//! `cargo test -p oxysol-tui print_ -- --nocapture` to read the output.

use oxysol_core::{Rank, Suit};

use super::board::{KLONDIKE_THREE, SPIDER, board};
use super::*;
use crate::test_support::{PLAIN, TRUE_COLOR};

fn print_cards(title: &str, cards: &[Card]) {
    each_locale(|locale| {
        for mode in MODES {
            for size in SIZES {
                println!("== {title} {locale:?} {mode:?} {}x{}", size.0, size.1);
                let renders: Vec<_> = cards.iter().map(|c| render(mode, size, *c)).collect();
                for row in 0..usize::from(size.1) {
                    let line: Vec<&str> = renders.iter().map(|r| r[row].as_str()).collect();
                    println!("{}", line.join(" "));
                }
            }
        }
    });
}

#[test]
fn print_every_suit_and_every_rank_at_every_size() {
    let suits: Vec<_> = Suit::ALL.map(|suit| card(Rank::Ace, suit)).into();
    print_cards("suits", &suits);
    for suit in Suit::ALL {
        let ranks: Vec<_> = Rank::ALL.map(|rank| card(rank, suit)).into();
        print_cards(&format!("ranks of {suit:?}"), &ranks);
    }
}

#[test]
fn print_the_ranks_that_were_confused_in_the_pixel_font() {
    let ranks: Vec<_> = [Rank::Ten, Rank::Jack, Rank::Queen, Rank::King]
        .map(|rank| card(rank, Suit::Diamonds))
        .into();
    print_cards("ten jack queen king", &ranks);
}

#[test]
fn print_klondike_and_spider_boards() {
    for size in [(100, 30), (130, 40)] {
        for (name, game) in [("Klondike", (KLONDIKE_THREE, 42)), ("Spider", (SPIDER, 42))] {
            for (caps, mode) in [(TRUE_COLOR, "Unicode"), (PLAIN, "ASCII")] {
                for locale in Locale::offered() {
                    let mut h = board(game, caps, locale, size);
                    if name == "Klondike" {
                        h.type_char('s');
                    }
                    println!("== {name} {mode} {locale:?} {}x{}", size.0, size.1);
                    for line in h.lines() {
                        println!("{}", line.trim_end());
                    }
                }
            }
        }
    }
    set_locale(Locale::EnUs);
}

const SNAPSHOT_FRONTS: [&str; 6] = [
    "french",
    "german",
    "international",
    "italian",
    "spanish",
    "swiss",
];
const SNAPSHOT_SIZES: [(u16, u16); 2] = [(11, 9), (13, 11)];

fn snapshot_row(
    fixture: &Fixture,
    size: (u16, u16),
    cards: &[Card],
    covered: Option<u16>,
) -> Vec<String> {
    let renders: Vec<Vec<String>> = cards
        .iter()
        .map(|card| {
            let area = Rect::new(0, 0, size.0, size.1);
            let mut buf = Buffer::empty(area);
            match covered {
                Some(visible) => draw_covered(
                    &mut buf,
                    area,
                    visible,
                    (Face::Up(*card), Mark::default()),
                    &fixture.ctx(),
                ),
                None => draw(
                    &mut buf,
                    area,
                    Face::Up(*card),
                    Mark::default(),
                    None,
                    &fixture.ctx(),
                ),
            }
            rows(&buf, area)
        })
        .collect();
    (0..usize::from(size.1))
        .map(|row| {
            renders
                .iter()
                .map(|lines| lines[row].as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

#[test]
fn print_large_face_snapshots() {
    let cards = [
        card(Rank::King, Suit::Hearts),
        card(Rank::Seven, Suit::Hearts),
        card(Rank::King, Suit::Spades),
        card(Rank::Seven, Suit::Spades),
    ];
    for front in SNAPSHOT_FRONTS {
        let fixture = Fixture::styled(GlyphMode::Unicode, front, "lattice");
        for size in SNAPSHOT_SIZES {
            println!("== {front} {}x{} uncovered K♥ 7♥ K♠ 7♠", size.0, size.1);
            for line in snapshot_row(&fixture, size, &cards, None) {
                println!("{line}");
            }
            println!("== {front} {}x{} covered (4 rows visible)", size.0, size.1);
            for line in snapshot_row(&fixture, size, &cards, Some(4)) {
                println!("{line}");
            }
        }
    }
}
