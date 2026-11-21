//! Tests for card back rendering: the back styles, the card back palette, sizes and glyph modes.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::card_style::CardStyles;
use crate::layout::SIZES;
use crate::settings::CardBack;
use crate::widgets::card::back::pattern;

const COUNT: usize = 21;
const BADGE: &str = "x21";
const STOCK: usize = 24;
const ALL_MODES: [GlyphMode; 3] = [GlyphMode::Font, GlyphMode::Unicode, GlyphMode::Ascii];

fn shows(buf: &Buffer, area: Rect, needle: &str) -> bool {
    (area.top()..area.bottom()).any(|y| {
        let row: String = (area.left()..area.right())
            .map(|x| buf[(x, y)].symbol())
            .collect();
        row.contains(needle)
    })
}

fn back_ids() -> Vec<String> {
    CardStyles::embedded()
        .backs()
        .iter()
        .map(|style| style.id.clone())
        .collect()
}

fn mosaic_of(id: &str, mode: GlyphMode) -> [String; 2] {
    let rows = CardStyles::embedded()
        .back_or_default(id)
        .map(|style| style.mosaic_rows(mode));
    rows.unwrap_or_default()
}

fn stock_grid(id: &str, mode: GlyphMode, size: (u16, u16)) -> Vec<String> {
    let fixture = Fixture::styled(mode, "international", id);
    let area = Rect::new(0, 0, size.0, size.1);
    let mut buf = Buffer::empty(area);
    draw(
        &mut buf,
        area,
        Face::Down,
        Mark::default(),
        Some(STOCK),
        &fixture.ctx(),
    );
    rows(&buf, area)
}

#[test]
fn every_back_style_at_every_card_size_in_every_glyph_mode_shows_its_tile_and_count_mark() {
    for id in back_ids() {
        for size in SIZES {
            for mode in ALL_MODES {
                let fixture = Fixture::styled(mode, "international", &id);
                let ctx = fixture.ctx();
                let tile = pattern(&fixture.glyphs);
                let area = Rect::new(0, 0, size.width, size.height);
                let case = format!("{id} {size:?} {mode:?}");

                let mut buf = Buffer::empty(area);
                draw(
                    &mut buf,
                    area,
                    Face::Down,
                    Mark::default(),
                    Some(COUNT),
                    &ctx,
                );
                assert!(shows(&buf, area, BADGE), "{case}: count");
                assert!(shows(&buf, area, tile), "{case}: tile");

                let mut collapsed = Buffer::empty(area);
                draw_collapsed(&mut collapsed, area, COUNT, &ctx);
                assert!(shows(&collapsed, area, BADGE), "{case}: collapsed");

                let mut back_row = Buffer::empty(area);
                draw_back_row(&mut back_row, area, Mark::default(), &ctx);
                assert!(shows(&back_row, area, tile), "{case}: row");
            }
        }
    }
}

#[test]
fn no_back_style_draws_a_centre_mark_at_any_size_or_glyph_mode() {
    for id in back_ids() {
        for size in SIZES {
            for mode in ALL_MODES {
                let fixture = Fixture::styled(mode, "international", &id);
                let tile = pattern(&fixture.glyphs);
                let area = Rect::new(0, 0, size.width, size.height);
                let mut buf = Buffer::empty(area);
                draw(
                    &mut buf,
                    area,
                    Face::Down,
                    Mark::default(),
                    None,
                    &fixture.ctx(),
                );
                let case = format!("{id} {size:?} {mode:?}");
                for y in area.top() + 1..area.bottom() - 1 {
                    for x in area.left() + 1..area.right() - 1 {
                        assert_eq!(buf[(x, y)].symbol(), tile, "{case}: cell {x},{y}");
                    }
                }
            }
        }
    }
}

#[test]
fn the_stock_back_is_the_tile_and_the_count_with_no_centre_mark_in_the_text_grid() {
    assert_eq!(
        stock_grid("lattice", GlyphMode::Unicode, (11, 9)),
        [
            "┌─────────┐",
            "│▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒│",
            "└──────x24┘",
        ]
    );
    assert_eq!(
        stock_grid("lattice", GlyphMode::Unicode, (13, 11)),
        [
            "┌───────────┐",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "│▒▒▒▒▒▒▒▒▒▒▒│",
            "└────────x24┘",
        ]
    );
    assert_eq!(
        stock_grid("ornament", GlyphMode::Ascii, (11, 9)),
        [
            "+---------+",
            "|%%%%%%%%%|",
            "|%%%%%%%%%|",
            "|%%%%%%%%%|",
            "|%%%%%%%%%|",
            "|%%%%%%%%%|",
            "|%%%%%%%%%|",
            "|%%%%%%%%%|",
            "+------x24+",
        ]
    );
}

#[test]
fn the_back_styles_have_different_tiles_and_mosaics_in_every_glyph_mode() {
    for mode in ALL_MODES {
        let tiles: Vec<String> = back_ids()
            .iter()
            .map(|id| pattern(&Fixture::styled(mode, "international", id).glyphs).to_owned())
            .collect();
        let mosaics: Vec<[String; 2]> = back_ids().iter().map(|id| mosaic_of(id, mode)).collect();
        assert_eq!(tiles.len(), 2);
        assert_ne!(tiles[0], tiles[1], "{mode:?}");
        assert_ne!(mosaics[0], mosaics[1], "{mode:?}");
    }
}

#[test]
fn the_card_back_setting_changes_the_colors_and_not_the_pattern() {
    for mode in ALL_MODES {
        let render = |back: CardBack| {
            let mut fixture = Fixture::styled(mode, "international", "ornament");
            fixture.settings.card_back = back;
            let area = Rect::new(0, 0, 11, 9);
            let mut buf = Buffer::empty(area);
            draw(
                &mut buf,
                area,
                Face::Down,
                Mark::default(),
                None,
                &fixture.ctx(),
            );
            buf
        };
        let theme = render(CardBack::Theme);
        let monokai = render(CardBack::Monokai);
        let dracula = render(CardBack::Dracula);
        let area = Rect::new(0, 0, 11, 9);
        assert_eq!(rows(&theme, area), rows(&monokai, area), "{mode:?}");
        assert_eq!(rows(&theme, area), rows(&dracula, area), "{mode:?}");
        assert_ne!(theme[(5, 2)].style(), monokai[(5, 2)].style(), "{mode:?}");
        assert_ne!(monokai[(5, 2)].style(), dracula[(5, 2)].style(), "{mode:?}");
    }
}

#[test]
fn an_unknown_back_style_draws_the_default_back() {
    let unknown = Fixture::styled(GlyphMode::Unicode, "international", "removed");
    let default = Fixture::styled(GlyphMode::Unicode, "international", "lattice");
    assert_eq!(unknown.glyphs, default.glyphs);
}
