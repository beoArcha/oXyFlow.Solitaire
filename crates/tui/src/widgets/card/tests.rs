use std::collections::HashSet;
use std::time::Instant;

use oxysol_core::{Card, CardId, Rank, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use super::art::{Scale, suit_art};
use super::*;
use crate::glyphs::{Glyphs, is_wgl4};
use crate::screens::ViewCtx;
use crate::settings::{GlyphMode, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};

const SIZES: [(u16, u16); 4] = [(13, 11), (11, 9), (9, 7), (7, 5)];
const MARGIN: u16 = 2;
const SENTINEL: &str = "?";

struct Fixture {
    theme: Theme,
    glyphs: Glyphs,
    settings: Settings,
}

impl Fixture {
    fn new(mode: GlyphMode) -> Self {
        Self {
            theme: Theme::new(ThemeId::Dark, TRUE_COLOR.color),
            glyphs: Glyphs::new(mode),
            settings: Settings::default(),
        }
    }

    fn ctx(&self) -> ViewCtx<'_> {
        ViewCtx {
            theme: &self.theme,
            glyphs: &self.glyphs,
            settings: &self.settings,
            caps: &TRUE_COLOR,
            audio_available: false,
            has_saved_game: false,
            now: Instant::now(),
        }
    }
}

fn card(rank: Rank, suit: Suit) -> Card {
    Card {
        id: CardId(0),
        suit,
        rank,
    }
}

fn padded(mode: GlyphMode, size: (u16, u16), card: Card) -> (Buffer, Rect) {
    let fixture = Fixture::new(mode);
    let whole = Rect::new(0, 0, size.0 + 2 * MARGIN, size.1 + 2 * MARGIN);
    let area = Rect::new(MARGIN, MARGIN, size.0, size.1);
    let mut buf = Buffer::empty(whole);
    for y in whole.top()..whole.bottom() {
        for x in whole.left()..whole.right() {
            buf[(x, y)].set_symbol(SENTINEL);
        }
    }
    draw(
        &mut buf,
        area,
        Face::Up(card),
        Mark::default(),
        None,
        &fixture.ctx(),
    );
    (buf, area)
}

fn rows(buf: &Buffer, area: Rect) -> Vec<String> {
    (area.top()..area.bottom())
        .map(|y| {
            (area.left()..area.right())
                .map(|x| buf[(x, y)].symbol())
                .collect()
        })
        .collect()
}

fn render(mode: GlyphMode, size: (u16, u16), card: Card) -> Vec<String> {
    let (buf, area) = padded(mode, size, card);
    rows(&buf, area)
}

fn interior(lines: &[String]) -> Vec<String> {
    let last = lines.len() - 1;
    lines[1..last]
        .iter()
        .map(|line| {
            line.chars()
                .skip(1)
                .take(line.chars().count() - 2)
                .collect()
        })
        .collect()
}

fn print_group(title: &str, samples: &[Card]) {
    for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
        for size in SIZES {
            println!("== {title} {mode:?} {}x{}", size.0, size.1);
            let cards: Vec<Vec<String>> = samples
                .iter()
                .map(|sample| render(mode, size, *sample))
                .collect();
            for row in 0..usize::from(size.1) {
                let line: Vec<&str> = cards.iter().map(|c| c[row].as_str()).collect();
                println!("{}", line.join(" "));
            }
        }
    }
}

#[test]
fn print_sample_renders() {
    let suits = Suit::ALL.map(|suit| card(Rank::Ace, suit));
    let ranks = [
        card(Rank::Seven, Suit::Hearts),
        card(Rank::Ten, Suit::Diamonds),
        card(Rank::Jack, Suit::Clubs),
        card(Rank::Queen, Suit::Hearts),
        card(Rank::King, Suit::Spades),
    ];
    print_group("suits", &suits);
    print_group("ranks", &ranks);
}

#[test]
fn every_art_row_has_the_width_and_height_of_its_scale() {
    for (scale, suit, unicode) in all_art() {
        let art = suit_art(scale, suit, unicode);
        assert_eq!(art.len(), usize::from(scale.suit_height()));
        for row in art {
            assert_eq!(row.chars().count(), usize::from(scale.suit_width()));
        }
    }
}

fn all_art() -> Vec<(Scale, Suit, bool)> {
    let mut all = Vec::new();
    for scale in [Scale::Large, Scale::Medium, Scale::Small] {
        for suit in Suit::ALL {
            for unicode in [true, false] {
                all.push((scale, suit, unicode));
            }
        }
    }
    all
}

#[test]
fn unicode_art_is_wgl4_and_ascii_art_is_ascii() {
    for (scale, suit, unicode) in all_art() {
        for row in suit_art(scale, suit, unicode) {
            for symbol in row.chars() {
                if unicode {
                    assert!(is_wgl4(symbol), "{symbol:?} in {scale:?} {suit:?}");
                } else {
                    assert!(symbol.is_ascii(), "{symbol:?} in {scale:?} {suit:?}");
                }
            }
        }
    }
}

type Case = (GlyphMode, (u16, u16), Card);

fn all_faces() -> Vec<Case> {
    let mut all = Vec::new();
    for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
        for size in SIZES {
            for rank in Rank::ALL {
                all.extend(Suit::ALL.map(|suit| (mode, size, card(rank, suit))));
            }
        }
    }
    all
}

#[test]
fn every_drawn_face_is_wgl4_or_ascii_for_every_size_rank_and_suit() {
    for (mode, size, face) in all_faces() {
        let (buf, area) = padded(mode, size, face);
        for symbol in rows(&buf, area).iter().flat_map(|line| line.chars()) {
            let allowed = match mode {
                GlyphMode::Ascii => symbol.is_ascii(),
                _ => is_wgl4(symbol),
            };
            assert!(allowed, "{symbol:?} {mode:?} {size:?}");
        }
    }
}

fn assert_only_the_card_is_painted(buf: &Buffer, area: Rect) {
    let whole = buf.area;
    for y in whole.top()..whole.bottom() {
        for x in whole.left()..whole.right() {
            let inside = area.contains(Position::new(x, y));
            assert_eq!(inside, buf[(x, y)].symbol() != SENTINEL, "({x},{y})");
        }
    }
}

#[test]
fn the_face_stays_inside_the_card_border_for_every_size() {
    for (mode, size, face) in all_faces() {
        let vertical = if mode == GlyphMode::Ascii {
            '|'
        } else {
            '\u{2502}'
        };
        let (buf, area) = padded(mode, size, face);
        assert_only_the_card_is_painted(&buf, area);
        let lines = rows(&buf, area);
        for line in &lines[1..lines.len() - 1] {
            assert_eq!(line.chars().next(), Some(vertical), "{size:?} {line}");
            assert_eq!(line.chars().last(), Some(vertical), "{size:?} {line}");
        }
    }
}

#[test]
fn the_four_suits_draw_four_different_shapes_at_every_size() {
    for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
        for size in SIZES {
            let shapes: HashSet<Vec<String>> = Suit::ALL
                .iter()
                .map(|suit| interior(&render(mode, size, card(Rank::Five, *suit))))
                .collect();
            assert_eq!(shapes.len(), 4, "{mode:?} {size:?}");
        }
    }
    for scale in [Scale::Large, Scale::Medium, Scale::Small] {
        for unicode in [true, false] {
            let shapes: HashSet<_> = Suit::ALL
                .iter()
                .map(|suit| suit_art(scale, *suit, unicode))
                .collect();
            assert_eq!(shapes.len(), 4, "{scale:?} {unicode}");
        }
    }
}

#[test]
fn every_rank_draws_a_different_large_centre() {
    for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
        for size in &SIZES[..3] {
            let shapes: HashSet<Vec<String>> = Rank::ALL
                .iter()
                .map(|rank| interior(&render(mode, *size, card(*rank, Suit::Spades))))
                .collect();
            assert_eq!(shapes.len(), 13, "{mode:?} {size:?}");
        }
    }
}

#[test]
fn ten_fits_the_interior_of_every_large_size() {
    for unicode in [true, false] {
        let Some(rows) = rank_font::render("10", unicode) else {
            unreachable!("both digits have a shape")
        };
        assert_eq!(rows.len(), usize::from(rank_font::HEIGHT));
        for row in rows {
            assert!(row.chars().count() <= 7, "{row}");
        }
    }
}

#[test]
fn the_compact_size_keeps_one_suit_symbol_and_no_art() {
    let lines = render(GlyphMode::Unicode, (7, 5), card(Rank::Ace, Suit::Hearts));
    let text = lines.join("\n");
    assert_eq!(text.matches('♥').count(), 3, "{text}");
    assert!(!text.contains('█') && !text.contains('▀') && !text.contains('▄'));
}

#[test]
fn the_pictogram_uses_the_suit_color_token() {
    let fixture = Fixture::new(GlyphMode::Unicode);
    for (suit, token) in [
        (Suit::Hearts, Token::CardRed),
        (Suit::Spades, Token::CardBlack),
    ] {
        let (buf, area) = padded(GlyphMode::Unicode, (13, 11), card(Rank::Ace, suit));
        let expected = fixture.theme.style(token, Token::CardFace);
        let painted: Vec<_> = (area.top()..area.bottom())
            .flat_map(|y| (area.left()..area.right()).map(move |x| (x, y)))
            .filter(|(x, y)| matches!(buf[(*x, *y)].symbol(), "█" | "▀" | "▄"))
            .collect();
        assert!(painted.len() > 10);
        for (x, y) in painted {
            assert_eq!(buf[(x, y)].fg, expected.fg.unwrap_or_default());
        }
    }
}
