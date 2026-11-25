//! Covered and blocked card rendering: no clipped centre pips at any card size or glyph mode.

use oxysol_core::{Rank, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::card_style::{FONT_FIRST, FONT_LAST};
use crate::widgets::card::art::{Scale, suit_art};

const ALL_MODES: [GlyphMode; 3] = [GlyphMode::Font, GlyphMode::Unicode, GlyphMode::Ascii];

fn sample_cards() -> [Card; 4] {
    [
        card(Rank::Ace, Suit::Clubs),
        card(Rank::Ten, Suit::Hearts),
        card(Rank::Queen, Suit::Diamonds),
        card(Rank::King, Suit::Spades),
    ]
}

fn pip_scale(size: (u16, u16)) -> Option<Scale> {
    match size {
        (13, 11) => Some(Scale::Large),
        (11, 9) => Some(Scale::Medium),
        (9, 7) => Some(Scale::Small),
        _ => None,
    }
}

fn pip_bottom(size: (u16, u16)) -> u16 {
    match size {
        (13, 11) => 8,
        (11, 9) => 6,
        (9, 7) => 5,
        _ => 3,
    }
}

fn inner_row(line: &str) -> String {
    let len = line.chars().count();
    line.chars().skip(1).take(len.saturating_sub(2)).collect()
}

fn is_pua(ch: char) -> bool {
    (FONT_FIRST..=FONT_LAST).contains(&u32::from(ch))
}

fn assert_pip_present(lines: &[String], mode: GlyphMode, size: (u16, u16), c: Card, what: &str) {
    let body = &lines[2..lines.len() - 1];
    match (mode, pip_scale(size)) {
        (GlyphMode::Font, Some(_)) => {
            assert!(
                body.iter().any(|row| row.chars().any(is_pua)),
                "{what}: expected font centre glyph in {lines:?}"
            );
        }
        (_, Some(scale)) => {
            let art = suit_art(scale, c.suit, mode != GlyphMode::Ascii);
            for art_row in art {
                assert!(
                    body.iter().any(|row| row.contains(art_row)),
                    "{what}: missing art row {art_row:?} in {lines:?}"
                );
            }
        }
        (_, None) => {
            let glyphs = Glyphs::styled(mode, "international", "lattice");
            let suit = glyphs.suit(c.suit);
            assert!(
                lines[2].contains(suit),
                "{what}: missing {suit} in {lines:?}"
            );
        }
    }
}

fn assert_no_pip_below_header(lines: &[String], what: &str) {
    for (row_index, line) in lines.iter().enumerate().take(lines.len() - 1).skip(2) {
        let inside = inner_row(line);
        assert!(
            inside.trim().is_empty(),
            "{what}: row {row_index} should be blank below header, got {line:?}"
        );
    }
}

fn assert_header_readable(
    lines: &[String],
    glyphs: &Glyphs,
    size: (u16, u16),
    visible_rows: u16,
    c: Card,
    what: &str,
) {
    let corner = label(c, glyphs);
    let top_after_corner: String = lines[0].chars().skip(1).collect();
    assert!(
        top_after_corner.starts_with(&corner),
        "{what}: corner {corner:?} missing in {:?}",
        lines[0]
    );
    if visible_rows >= 2 && size.0 >= 9 {
        let rank = rank_in(c.rank, glyphs);
        let suit = glyphs.suit(c.suit);
        assert!(
            lines[1].contains(&rank) && lines[1].contains(suit),
            "{what}: header {rank} {suit} missing in {:?}",
            lines[1]
        );
    }
}

fn check_covered_case(mode: GlyphMode, size: (u16, u16), c: Card, visible: u16) {
    let fixture = Fixture::styled(mode, "international", "lattice");
    let area = Rect::new(0, 0, size.0, size.1);
    let mut buf = Buffer::empty(area);
    draw_covered(
        &mut buf,
        area,
        visible,
        (Face::Up(c), Mark::default()),
        &fixture.ctx(),
    );
    let lines = rows(&buf, area);
    let what = format!("{mode:?} {size:?} {c:?} visible={visible}");
    assert_header_readable(&lines, &fixture.glyphs, size, visible, c, &what);
    if visible < pip_bottom(size) {
        assert_no_pip_below_header(&lines, &what);
    } else {
        assert_pip_present(&lines, mode, size, c, &what);
    }
}

#[test]
fn covered_card_omits_the_centre_pip_unless_the_whole_pip_area_fits() {
    for mode in ALL_MODES {
        for size in SIZES {
            for c in sample_cards() {
                for visible in 1..size.1 {
                    check_covered_case(mode, size, c, visible);
                }
            }
        }
    }
}

#[test]
fn blocked_card_draws_header_only_and_no_centre_pip_in_every_mode_and_size() {
    let mark = Mark {
        blocked: true,
        ..Mark::default()
    };
    for mode in ALL_MODES {
        let fixture = Fixture::styled(mode, "international", "lattice");
        for size in SIZES {
            for c in sample_cards() {
                let area = Rect::new(0, 0, size.0, size.1);
                let mut buf = Buffer::empty(area);
                draw(&mut buf, area, Face::Up(c), mark, None, &fixture.ctx());
                let lines = rows(&buf, area);
                let what = format!("blocked {mode:?} {size:?} {c:?}");
                assert_header_readable(&lines, &fixture.glyphs, size, size.1, c, &what);
                assert_no_pip_below_header(&lines, &what);
            }
        }
    }
}

#[test]
fn uncovered_card_still_draws_its_full_centre_pip_in_every_mode_and_size() {
    for mode in ALL_MODES {
        let fixture = Fixture::styled(mode, "international", "lattice");
        for size in SIZES {
            for c in sample_cards() {
                let area = Rect::new(0, 0, size.0, size.1);
                let mut buf = Buffer::empty(area);
                draw(
                    &mut buf,
                    area,
                    Face::Up(c),
                    Mark::default(),
                    None,
                    &fixture.ctx(),
                );
                let lines = rows(&buf, area);
                let what = format!("uncovered {mode:?} {size:?} {c:?}");
                assert_header_readable(&lines, &fixture.glyphs, size, size.1, c, &what);
                assert_pip_present(&lines, mode, size, c, &what);
            }
        }
    }
}
