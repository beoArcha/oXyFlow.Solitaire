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
fn the_mosaic_is_drawn_only_at_the_large_and_huge_sizes() {
    for id in back_ids() {
        for mode in ALL_MODES {
            let fixture = Fixture::styled(mode, "international", &id);
            let [top, bottom] = mosaic_of(&id, mode);
            for size in SIZES {
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
                let large = size.width >= 11 && size.height >= 9;
                let case = format!("{id} {size:?} {mode:?}");
                assert_eq!(shows(&buf, area, &top), large, "{case}: top row");
                assert_eq!(shows(&buf, area, &bottom), large, "{case}: bottom row");
            }
        }
    }
}

#[test]
fn the_mosaic_sits_in_the_middle_of_a_large_back_as_two_rows_of_two_symbols() {
    let fixture = Fixture::styled(GlyphMode::Unicode, "international", "lattice");
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
    let lines = rows(&buf, area);
    assert_eq!(lines[3], "│▒▒▒▓▒▒▒▒▒│");
    assert_eq!(lines[4], "│▒▒▒▒▓▒▒▒▒│");
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

#[test]
fn the_font_mosaic_needs_a_cell_of_ten_by_twenty_four_pixels_where_the_cell_size_is_known() {
    let area = Rect::new(0, 0, 11, 9);
    let [top, _] = mosaic_of("lattice", GlyphMode::Font);
    let drawn = |cell_px: Option<(u16, u16)>, mode: GlyphMode| {
        let fixture = Fixture::styled(mode, "international", "lattice");
        let mut ctx = fixture.ctx();
        ctx.font.cell_px = cell_px;
        let mut buf = Buffer::empty(area);
        draw(&mut buf, area, Face::Down, Mark::default(), None, &ctx);
        buf
    };
    for (cell, expected) in [
        (None, true),
        (Some((10, 24)), true),
        (Some((12, 30)), true),
        (Some((9, 24)), false),
        (Some((10, 23)), false),
        (Some((8, 16)), false),
    ] {
        let buf = drawn(cell, GlyphMode::Font);
        assert_eq!(shows(&buf, area, &top), expected, "{cell:?}");
    }
    let [unicode_top, _] = mosaic_of("lattice", GlyphMode::Unicode);
    let small = drawn(Some((8, 16)), GlyphMode::Unicode);
    assert!(shows(&small, area, &unicode_top));
}
