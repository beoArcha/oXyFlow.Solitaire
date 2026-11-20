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
