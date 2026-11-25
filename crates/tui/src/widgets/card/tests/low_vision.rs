//! Card widget tests for low-vision mode: bold rank & suit, no dim text on blocked cards,
//! larger focus mark (`►►` / `>>`), and four-corner / wide empty target marks.

use oxysol_core::{Rank, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::*;
use crate::settings::{GlyphMode, Settings};

fn low_vision_fixture(mode: GlyphMode) -> Fixture {
    Fixture {
        theme: Theme::new(ThemeId::HighContrast, TRUE_COLOR.color).with_low_vision(true),
        glyphs: Glyphs::new(mode),
        settings: Settings {
            low_vision: true,
            ..Settings::default()
        },
    }
}

#[test]
fn low_vision_makes_rank_suit_and_focus_border_bold_and_never_dims_blocked_cards() {
    for mode in MODES {
        let fixture = low_vision_fixture(mode);
        let ctx = fixture.ctx();
        let area = Rect::new(0, 0, 9, 7);
        let mut buf = Buffer::empty(area);
        let blocked = Mark {
            blocked: true,
            ..Mark::default()
        };
        draw(
            &mut buf,
            area,
            Face::Up(card(Rank::Queen, Suit::Hearts)),
            blocked,
            None,
            &ctx,
        );
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                assert!(
                    !buf[(x, y)].modifier.contains(Modifier::DIM),
                    "{mode:?} ({x},{y}) must not be DIM"
                );
            }
        }
        assert!(buf[(1, 0)].modifier.contains(Modifier::BOLD));
        assert!(buf[(2, 0)].modifier.contains(Modifier::BOLD));
        assert!(buf[(4, 3)].modifier.contains(Modifier::BOLD));
    }
}

#[test]
fn low_vision_enlarges_focus_and_target_marks_without_relying_on_color() {
    for mode in MODES {
        let fixture = low_vision_fixture(mode);
        let ctx = fixture.ctx();
        let area = Rect::new(0, 0, 9, 7);

        let mut focus_buf = Buffer::empty(area);
        draw(
            &mut focus_buf,
            area,
            Face::Up(card(Rank::Ace, Suit::Spades)),
            Mark {
                focus: true,
                ..Mark::default()
            },
            None,
            &ctx,
        );
        let top: String = (0..area.width)
            .map(|x| focus_buf[(x, 0)].symbol())
            .collect();
        let doubled = if mode == GlyphMode::Unicode { "►►" } else { ">>" };
        assert!(top.contains(doubled), "{mode:?}: {top}");

        let mut target_buf = Buffer::empty(area);
        draw(
            &mut target_buf,
            area,
            Face::Up(card(Rank::King, Suit::Diamonds)),
            Mark {
                target: true,
                ..Mark::default()
            },
            None,
            &ctx,
        );
        assert_eq!(target_buf[(0, 0)].symbol(), "+");
        assert_eq!(target_buf[(8, 0)].symbol(), "+");
        assert_eq!(target_buf[(0, 6)].symbol(), "+");
        assert_eq!(target_buf[(8, 6)].symbol(), "+");

        let mut slot_buf = Buffer::empty(area);
        draw(
            &mut slot_buf,
            area,
            Face::Slot(Placeholder::None),
            Mark {
                target: true,
                ..Mark::default()
            },
            None,
            &ctx,
        );
        let mid: String = (0..area.width)
            .map(|x| slot_buf[(x, 3)].symbol())
            .collect();
        assert!(mid.contains("+++"), "{mode:?}: {mid}");
    }
}
