//! Tests of the card fronts: every front style in every glyph mode at every card size.

use std::collections::HashSet;

use oxysol_core::{Rank, Suit};
use ratatui::style::Color;

use super::*;
use crate::card_style::{CardStyles, FONT_FIRST, FONT_LAST};
use crate::layout::SIZES;
use crate::theme::Token;

const ALL_MODES: [GlyphMode; 3] = [GlyphMode::Font, GlyphMode::Unicode, GlyphMode::Ascii];

fn front_ids() -> Vec<String> {
    CardStyles::embedded()
        .fronts()
        .iter()
        .map(|style| style.id.clone())
        .collect()
}

fn draw_card(fixture: &Fixture, size: (u16, u16), face: Face) -> Buffer {
    let area = Rect::new(0, 0, size.0, size.1);
    let mut buf = Buffer::empty(area);
    draw(&mut buf, area, face, Mark::default(), None, &fixture.ctx());
    buf
}

fn lines(buf: &Buffer) -> Vec<String> {
    rows(buf, buf.area)
}

fn card_lines(fixture: &Fixture, size: (u16, u16), card: Card) -> Vec<String> {
    lines(&draw_card(fixture, size, Face::Up(card)))
}

fn is_private_use(text: &str) -> bool {
    text.chars()
        .any(|c| (FONT_FIRST..=FONT_LAST).contains(&u32::from(c)))
}

#[test]
fn the_four_suits_of_every_front_differ_by_shape_in_every_mode_and_size() {
    for id in front_ids() {
        for mode in ALL_MODES {
            let fixture = Fixture::styled(mode, &id, "lattice");
            for size in SIZES {
                let shapes: HashSet<Vec<String>> = Suit::ALL
                    .iter()
                    .map(|suit| {
                        card_lines(&fixture, (size.width, size.height), card(Rank::Five, *suit))
                    })
                    .collect();
                assert_eq!(shapes.len(), 4, "{id} {mode:?} {size:?}");
            }
        }
    }
}

#[test]
fn the_rank_is_readable_at_the_smallest_size_in_every_front_and_mode() {
    for id in front_ids() {
        for mode in ALL_MODES {
            let fixture = Fixture::styled(mode, &id, "lattice");
            for face in every_card() {
                let lines = card_lines(&fixture, (7, 5), face);
                let corner = label(face, &fixture.glyphs);
                let top: String = lines[0].chars().skip(1).collect();
                assert!(
                    top.starts_with(&corner),
                    "{id} {mode:?} {corner}: {}",
                    lines[0]
                );
                assert!(lines[4].contains(&corner), "{id} {mode:?} {corner}");
            }
        }
    }
}

#[test]
fn no_two_cards_render_the_same_in_any_front_mode_and_size() {
    each_locale(|locale| {
        for id in front_ids() {
            for mode in ALL_MODES {
                let fixture = Fixture::styled(mode, &id, "lattice");
                for size in SIZES {
                    let renders: HashSet<_> = every_card()
                        .iter()
                        .map(|face| card_lines(&fixture, (size.width, size.height), *face))
                        .collect();
                    assert_eq!(renders.len(), 52, "{locale:?} {id} {mode:?} {size:?}");
                }
            }
        }
    });
}

#[test]
fn private_use_glyphs_are_drawn_only_in_font_mode() {
    for id in front_ids() {
        for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
            let fixture = Fixture::styled(mode, &id, "ornament");
            for size in SIZES {
                for face in [
                    card(Rank::King, Suit::Hearts),
                    card(Rank::Four, Suit::Clubs),
                ] {
                    let drawn = card_lines(&fixture, (size.width, size.height), face).join("");
                    assert!(!is_private_use(&drawn), "{id} {mode:?} {size:?}");
                }
                let back = lines(&draw_card(&fixture, (size.width, size.height), Face::Down));
                assert!(
                    !is_private_use(&back.join("")),
                    "{id} {mode:?} {size:?} back"
                );
            }
        }
    }
}

#[test]
fn font_mode_draws_the_suit_in_the_corners_the_centre_and_the_court_emblem() {
    for id in front_ids() {
        let fixture = Fixture::styled(GlyphMode::Font, &id, "lattice");
        let Some(front) = fixture.glyphs.front else {
            panic!("{id} has no style");
        };
        if !front.has_font() {
            continue;
        }
        for suit in Suit::ALL {
            let symbol = front.suit_glyph(suit).unwrap_or_default();
            assert!(is_private_use(symbol), "{id} {suit:?}");
            let ace = card_lines(&fixture, (11, 9), card(Rank::Ace, suit));
            assert!(
                ace[0].contains(symbol) && ace[8].contains(symbol),
                "{id} corners"
            );
            assert!(
                ace[3..6].iter().any(|line| line.contains(symbol)),
                "{id} centre"
            );
            let queen = card_lines(&fixture, (11, 9), card(Rank::Queen, suit));
            let emblem = front.court_emblem(Rank::Queen).unwrap_or_default();
            assert!(is_private_use(emblem), "{id} emblem");
            assert!(
                queen[3..6].iter().any(|line| line.contains(emblem)),
                "{id} {suit:?}"
            );
        }
        let jack = front.court_emblem(Rank::Jack);
        let queen = front.court_emblem(Rank::Queen);
        let king = front.court_emblem(Rank::King);
        assert!(
            jack != queen && queen != king && jack != king,
            "{id} emblems differ"
        );
    }
}

fn assert_fontless_front_draws_its_unicode_symbols(id: &str) {
    let font = Fixture::styled(GlyphMode::Font, id, "lattice");
    let unicode = Fixture::styled(GlyphMode::Unicode, id, "lattice");
    let Some(front) = font.glyphs.front else {
        panic!("{id} has no style");
    };
    for size in SIZES {
        let size = (size.width, size.height);
        for face in every_card() {
            let drawn = card_lines(&font, size, face);
            assert!(!is_private_use(&drawn.join("")), "{face:?} {size:?}");
            assert_eq!(drawn, card_lines(&unicode, size, face), "{face:?} {size:?}");
        }
    }
    for (suit, symbol) in Suit::ALL
        .into_iter()
        .zip(front.suit_symbols(GlyphMode::Font))
    {
        let ace = card_lines(&font, (11, 9), card(Rank::Ace, suit));
        assert!(
            ace[0].contains(symbol) && ace[8].contains(symbol),
            "{suit:?}"
        );
    }
}

#[test]
fn swiss_in_font_mode_draws_its_unicode_symbols_and_renders_like_unicode_mode() {
    assert_fontless_front_draws_its_unicode_symbols("swiss");
}

#[test]
fn spanish_in_font_mode_draws_its_unicode_symbols_and_renders_like_unicode_mode() {
    assert_fontless_front_draws_its_unicode_symbols("spanish");
}

#[test]
fn swiss_cards_at_the_smallest_size_match_the_snapshot() {
    let fixture = Fixture::styled(GlyphMode::Unicode, "swiss", "lattice");
    assert_eq!(
        card_lines(&fixture, (7, 5), card(Rank::Ace, Suit::Hearts)),
        ["┌A☼───┐", "│     │", "│  ☼  │", "│     │", "└───A☼┘"]
    );
    let ascii = Fixture::styled(GlyphMode::Ascii, "swiss", "lattice");
    assert_eq!(
        card_lines(&ascii, (7, 5), card(Rank::King, Suit::Clubs)),
        ["+Ke---+", "|     |", "|  e  |", "|     |", "+---Ke+"]
    );
}

#[test]
fn spanish_cards_at_the_smallest_size_match_the_snapshot() {
    let fixture = Fixture::styled(GlyphMode::Unicode, "spanish", "lattice");
    assert_eq!(
        card_lines(&fixture, (7, 5), card(Rank::Ace, Suit::Hearts)),
        ["┌A♥───┐", "│     │", "│  ♥  │", "│     │", "└───A♥┘"]
    );
    let ascii = Fixture::styled(GlyphMode::Ascii, "spanish", "lattice");
    assert_eq!(
        card_lines(&ascii, (7, 5), card(Rank::King, Suit::Clubs)),
        ["+Rb---+", "|     |", "|  b  |", "|     |", "+---Rb+"]
    );
}

#[test]
fn font_mode_without_font_glyphs_in_the_style_draws_the_unicode_symbols() {
    let plain = Glyphs::new(GlyphMode::Unicode);
    let font_without_style = Glyphs::new(GlyphMode::Font);
    assert_eq!(font_without_style, plain);
    assert!(!font_without_style.font);
}

#[test]
fn the_font_is_drawn_only_after_the_player_confirmed_it_or_the_console_shows_it() {
    let mut settings = Settings {
        glyph_mode: GlyphMode::Font,
        ..Settings::default()
    };
    assert_eq!(settings.drawn_glyph_mode(false), GlyphMode::Unicode);
    assert_eq!(settings.drawn_glyph_mode(true), GlyphMode::Font);
    settings.font_confirmed = true;
    assert_eq!(settings.drawn_glyph_mode(false), GlyphMode::Font);
    settings.glyph_mode = GlyphMode::Ascii;
    assert_eq!(settings.drawn_glyph_mode(true), GlyphMode::Ascii);
}

#[test]
fn the_court_labels_of_a_skin_replace_the_localized_ones() {
    let corner = |front: &str, mode: GlyphMode, rank: Rank, suit: Suit| {
        let fixture = Fixture::styled(mode, front, "lattice");
        let lines = card_lines(&fixture, (9, 7), card(rank, suit));
        lines[0].chars().skip(1).take(2).collect::<String>()
    };
    assert_eq!(
        corner("french", GlyphMode::Unicode, Rank::Jack, Suit::Hearts),
        "V♥"
    );
    assert_eq!(
        corner("french", GlyphMode::Ascii, Rank::Queen, Suit::Spades),
        "Ds"
    );
    assert_eq!(
        corner("german", GlyphMode::Ascii, Rank::King, Suit::Clubs),
        "Ka"
    );
    assert_eq!(
        corner("italian", GlyphMode::Ascii, Rank::Jack, Suit::Diamonds),
        "Fo"
    );
    assert_eq!(
        corner("international", GlyphMode::Ascii, Rank::Jack, Suit::Hearts),
        "Jh"
    );
}

#[test]
fn every_color_a_card_uses_is_the_color_of_a_token() {
    let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
    let allowed: HashSet<Color> = Token::ALL.iter().map(|token| theme.color(*token)).collect();
    for id in front_ids() {
        for mode in ALL_MODES {
            let fixture = Fixture::styled(mode, &id, "ornament");
            for size in SIZES {
                let size = (size.width, size.height);
                let mut faces = vec![Face::Down];
                faces.extend(every_card().into_iter().map(Face::Up));
                for face in faces {
                    let buf = draw_card(&fixture, size, face);
                    for cell in &buf.content {
                        assert!(allowed.contains(&cell.fg), "{id} {mode:?} {size:?} fg");
                        assert!(allowed.contains(&cell.bg), "{id} {mode:?} {size:?} bg");
                    }
                }
            }
        }
    }
}

#[test]
fn a_front_gives_its_suits_the_card_colors_of_the_theme() {
    let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
    for id in front_ids() {
        let fixture = Fixture::styled(GlyphMode::Unicode, &id, "lattice");
        for suit in Suit::ALL {
            let buf = draw_card(&fixture, (9, 7), Face::Up(card(Rank::Ace, suit)));
            let expected = theme.color(fixture.glyphs.suit_token(suit));
            assert_eq!(buf[(1, 0)].fg, expected, "{id} {suit:?}");
        }
    }
}

const LARGEST_SIZES: [(u16, u16); 2] = [(11, 9), (13, 11)];

#[test]
fn the_rank_stays_readable_in_the_corners_of_the_two_largest_cards_in_every_front_and_mode() {
    for id in front_ids() {
        for mode in ALL_MODES {
            let fixture = Fixture::styled(mode, &id, "lattice");
            for size in LARGEST_SIZES {
                for face in every_card() {
                    let lines = card_lines(&fixture, size, face);
                    let corner = label(face, &fixture.glyphs);
                    let case = format!("{id} {mode:?} {size:?} {corner}");
                    assert!(lines[0].contains(&corner), "{case}: {}", lines[0]);
                    assert!(lines[lines.len() - 1].contains(&corner), "{case}");
                }
            }
        }
    }
}

#[test]
fn a_covered_card_of_the_two_largest_sizes_keeps_its_rank_and_suit_in_the_corner() {
    for id in front_ids() {
        for mode in ALL_MODES {
            let fixture = Fixture::styled(mode, &id, "lattice");
            for size in LARGEST_SIZES {
                for face in every_card() {
                    for visible in 1..size.1 {
                        let area = Rect::new(0, 0, size.0, size.1);
                        let mut buf = Buffer::empty(area);
                        draw_covered(
                            &mut buf,
                            area,
                            visible,
                            (Face::Up(face), Mark::default()),
                            &fixture.ctx(),
                        );
                        let lines = rows(&buf, area);
                        let corner = label(face, &fixture.glyphs);
                        let case = format!("{id} {mode:?} {size:?} {corner} visible={visible}");
                        let top: String = lines[0].chars().skip(1).collect();
                        assert!(top.starts_with(&corner), "{case}: {}", lines[0]);
                    }
                }
            }
        }
    }
}
