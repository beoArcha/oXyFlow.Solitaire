use std::collections::HashSet;

use oxysol_core::{Rank, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;

use super::*;
use crate::glyphs::is_wgl4;
use crate::widgets::card::art::{Scale, suit_art};

type Case = (GlyphMode, (u16, u16), Card);

fn all_faces() -> Vec<Case> {
    let mut all = Vec::new();
    for mode in MODES {
        for size in SIZES {
            all.extend(every_card().into_iter().map(|face| (mode, size, face)));
        }
    }
    all
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
fn every_art_row_has_the_width_and_height_of_its_scale() {
    for (scale, suit, unicode) in all_art() {
        let art = suit_art(scale, suit, unicode);
        assert_eq!(art.len(), usize::from(scale.suit_height()));
        for row in art {
            assert_eq!(row.chars().count(), usize::from(scale.suit_width()));
        }
    }
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

#[test]
fn every_drawn_face_is_wgl4_or_ascii_for_every_size_rank_and_suit() {
    each_locale(|locale| {
        for (mode, size, face) in all_faces() {
            let (buf, area) = padded(mode, size, face);
            for symbol in rows(&buf, area).iter().flat_map(|line| line.chars()) {
                let allowed = match mode {
                    GlyphMode::Ascii => symbol.is_ascii(),
                    _ => is_wgl4(symbol),
                };
                assert!(allowed, "{symbol:?} {mode:?} {size:?} {locale:?}");
            }
        }
    });
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
    for mode in MODES {
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

fn centre_text(size: (u16, u16), rank: &str, suit: &str) -> String {
    let spaces = if size.0 >= 13 { "  " } else { " " };
    format!("{rank}{spaces}{suit}")
}

#[test]
fn the_corners_and_the_centre_carry_the_rank_and_the_suit_as_text() {
    each_locale(|locale| {
        for (mode, size, face) in all_faces() {
            let lines = render(mode, size, face);
            let glyphs = Glyphs::new(mode);
            let corner = label(face, &glyphs);
            let case = format!("{locale:?} {mode:?} {size:?} {corner}");
            let after_corner: String = lines[0].chars().skip(1).collect();
            assert!(after_corner.starts_with(&corner), "{case}: {}", lines[0]);
            assert!(lines[lines.len() - 1].contains(&corner), "{case}");
            let suit = glyphs.suit(face.suit);
            if size.0 >= 9 {
                let centre = centre_text(size, &rank_text(face.rank), suit);
                assert_eq!(lines[1].trim_matches([' ', '│', '|']), centre, "{case}");
            } else {
                assert!(lines[2].contains(suit), "{case}");
            }
        }
    });
}

#[test]
fn no_two_cards_render_the_same_at_any_size() {
    each_locale(|locale| {
        for mode in MODES {
            for size in SIZES {
                let renders: HashSet<_> = every_card()
                    .iter()
                    .map(|face| render(mode, size, *face))
                    .collect();
                assert_eq!(renders.len(), 52, "{locale:?} {mode:?} {size:?}");
                for suit in Suit::ALL {
                    let by_rank = Rank::ALL.map(|rank| render(mode, size, card(rank, suit)));
                    let corners: HashSet<_> = by_rank.iter().map(|lines| &lines[0]).collect();
                    let centres: HashSet<_> = by_rank.iter().map(|lines| &lines[1]).collect();
                    assert_eq!(corners.len(), 13, "{locale:?} {mode:?} {size:?} corners");
                    if size.0 >= 9 {
                        assert_eq!(centres.len(), 13, "{locale:?} {mode:?} {size:?} centres");
                    }
                }
            }
        }
    });
}

#[test]
fn the_polish_queen_of_diamonds_is_not_dd_in_ascii() {
    set_locale(Locale::PlPl);
    let glyphs = Glyphs::new(GlyphMode::Ascii);
    let queen = label(card(Rank::Queen, Suit::Diamonds), &glyphs);
    set_locale(Locale::EnUs);
    assert_eq!(queen, "Dd");
}

fn is_blank(line: &str) -> bool {
    line.trim_matches(['│', '|', ' ']).is_empty()
}

#[test]
fn the_text_row_is_followed_by_air_and_the_pictogram_without_a_pixel_rank() {
    let layouts = [((9, 7), 3, 2), ((11, 9), 3, 3), ((13, 11), 4, 4)];
    for mode in MODES {
        for (size, art_top, art_height) in layouts {
            let lines = render(mode, size, card(Rank::King, Suit::Spades));
            assert!(!is_blank(&lines[1]), "{size:?} text row");
            for row in 2..art_top {
                assert!(is_blank(&lines[row]), "{size:?} row {row}: {}", lines[row]);
            }
            for row in art_top..art_top + art_height {
                assert!(!is_blank(&lines[row]), "{size:?} row {row}");
            }
            for row in art_top + art_height..lines.len() - 1 {
                assert!(is_blank(&lines[row]), "{size:?} row {row}: {}", lines[row]);
            }
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
fn the_pictogram_uses_the_suit_color_token_and_the_text_is_bold() {
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
        let text_row = area.top() + 1;
        let letter = (area.left() + 1..area.right())
            .find(|x| buf[(*x, text_row)].symbol() == "A")
            .map(|x| &buf[(x, text_row)]);
        let Some(letter) = letter else {
            unreachable!("the centre text shows the rank")
        };
        assert!(letter.modifier.contains(Modifier::BOLD));
        assert_eq!(letter.fg, expected.fg.unwrap_or_default());
    }
}
