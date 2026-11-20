//! Tests for card back rendering, sizes, and glyph modes.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::layout::SIZES;
use crate::settings::CardBack;

#[test]
fn every_back_at_every_card_size_in_both_glyph_modes_renders_without_panic_and_with_count_mark_visible()
{
    let count = 21;
    let expected_badge = "x21";
    for back in CardBack::ALL {
        for size in SIZES {
            for mode in MODES {
                let mut fixture = Fixture::new(mode);
                fixture.settings.card_back = back;
                let ctx = fixture.ctx();
                let area = Rect::new(0, 0, size.width, size.height);
                let mut buf = Buffer::empty(area);

                draw(
                    &mut buf,
                    area,
                    Face::Down,
                    Mark::default(),
                    Some(count),
                    &ctx,
                );

                let mut found_badge = false;
                for y in area.top()..area.bottom() {
                    let mut row_text = String::new();
                    for x in area.left()..area.right() {
                        row_text.push_str(buf[(x, y)].symbol());
                    }
                    if row_text.contains(expected_badge) {
                        found_badge = true;
                        break;
                    }
                }
                assert!(
                    found_badge,
                    "Count mark {expected_badge} not found for {back:?} at {size:?} in {mode:?}"
                );

                let mut collapsed_buf = Buffer::empty(area);
                draw_collapsed(&mut collapsed_buf, area, count, &ctx);
                let mut found_collapsed = false;
                for y in area.top()..area.bottom() {
                    let mut row_text = String::new();
                    for x in area.left()..area.right() {
                        row_text.push_str(collapsed_buf[(x, y)].symbol());
                    }
                    if row_text.contains(expected_badge) {
                        found_collapsed = true;
                        break;
                    }
                }
                assert!(
                    found_collapsed,
                    "Collapsed count mark {expected_badge} not found for {back:?} at {size:?} in {mode:?}"
                );

                let mut back_row_buf = Buffer::empty(area);
                draw_back_row(&mut back_row_buf, area, Mark::default(), &ctx);
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
            .map(|back| crate::widgets::card::back::pattern(*back, &glyphs))
            .collect();
        assert_ne!(patterns[0], patterns[1], "mode {mode:?}");
        assert_ne!(patterns[0], patterns[2], "mode {mode:?}");
        assert_ne!(patterns[1], patterns[2], "mode {mode:?}");
    }
}
