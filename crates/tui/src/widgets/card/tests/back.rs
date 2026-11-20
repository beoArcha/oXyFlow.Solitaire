//! Tests for card back rendering, sizes, and glyph modes.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::layout::SIZES;
use crate::settings::CardBack;
use crate::widgets::card::back::pattern;

const COUNT: usize = 21;
const BADGE: &str = "x21";

fn shows(buf: &Buffer, area: Rect, needle: &str) -> bool {
    (area.top()..area.bottom()).any(|y| {
        let row: String = (area.left()..area.right())
            .map(|x| buf[(x, y)].symbol())
            .collect();
        row.contains(needle)
    })
}

#[test]
fn every_back_at_every_card_size_in_both_glyph_modes_shows_its_pattern_and_count_mark() {
    for back in CardBack::ALL {
        for size in SIZES {
            for mode in MODES {
                let mut fixture = Fixture::new(mode);
                fixture.settings.card_back = back;
                let ctx = fixture.ctx();
                let glyph = pattern(back, &Glyphs::new(mode));
                let area = Rect::new(0, 0, size.width, size.height);

                let mut buf = Buffer::empty(area);
                draw(
                    &mut buf,
                    area,
                    Face::Down,
                    Mark::default(),
                    Some(COUNT),
                    &ctx,
                );
                assert!(
                    shows(&buf, area, BADGE),
                    "{back:?} {size:?} {mode:?}: count"
                );
                assert!(
                    shows(&buf, area, glyph),
                    "{back:?} {size:?} {mode:?}: pattern"
                );

                let mut collapsed = Buffer::empty(area);
                draw_collapsed(&mut collapsed, area, COUNT, &ctx);
                assert!(
                    shows(&collapsed, area, BADGE),
                    "{back:?} {size:?} {mode:?}: collapsed"
                );

                let mut back_row = Buffer::empty(area);
                draw_back_row(&mut back_row, area, Mark::default(), &ctx);
                assert!(
                    shows(&back_row, area, glyph),
                    "{back:?} {size:?} {mode:?}: row"
                );
            }
        }
    }
}

#[test]
fn distinct_pattern_for_each_card_back_in_unicode_and_ascii() {
    for mode in MODES {
        let glyphs = Glyphs::new(mode);
        let patterns: Vec<&'static str> = CardBack::ALL
            .iter()
            .map(|back| pattern(*back, &glyphs))
            .collect();
        assert_ne!(patterns[0], patterns[1], "mode {mode:?}");
        assert_ne!(patterns[0], patterns[2], "mode {mode:?}");
        assert_ne!(patterns[1], patterns[2], "mode {mode:?}");
    }
}
