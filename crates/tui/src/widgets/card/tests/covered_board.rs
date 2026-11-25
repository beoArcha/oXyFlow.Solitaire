//! Board-level and pile-level tests verifying covered cards never show clipped centre pips in
//! FreeCell, Yukon, Pyramid, and the other games at every card size and glyph mode.

use std::time::Instant;

use oxysol_core::{Card, Game, PileRef, PileSnapshot, PyramidRules, Variant, YukonRules, deal};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::audio::AudioStatus;
use crate::card_style::{FONT_FIRST, FONT_LAST};
use crate::glyphs::Glyphs;
use crate::layout::{self, BoardGeometry, CardSize, SIZES};
use crate::screens::ViewCtx;
use crate::settings::{CardSizeSetting, GlyphMode, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};
use crate::widgets::card::{label, rank_in};
use crate::widgets::pile::{Marks, draw as draw_pile};

pub(super) const ALL_MODES: [GlyphMode; 3] =
    [GlyphMode::Font, GlyphMode::Unicode, GlyphMode::Ascii];
pub(super) const THREE_GAMES: [Variant; 3] = [
    Variant::FreeCell,
    Variant::Yukon(YukonRules::Classic),
    Variant::Pyramid(PyramidRules::Relaxed),
];

pub(super) fn settings_for(mode: GlyphMode, card_size: CardSizeSetting) -> Settings {
    let caps = if mode == GlyphMode::Ascii {
        crate::test_support::PLAIN
    } else {
        TRUE_COLOR
    };
    Settings {
        glyph_mode: mode,
        font_confirmed: mode == GlyphMode::Font,
        card_size,
        ..Settings::for_caps(&caps)
    }
}

fn pip_bounds(size: CardSize) -> (u16, u16) {
    match (size.width, size.height) {
        (13, 11) => (3, 7),
        (11, 9) => (2, 6),
        (9, 7) => (3, 5),
        _ => (2, 3),
    }
}

fn row_slice(buf: &Buffer, rect: Rect, row: u16) -> String {
    (rect.x..rect.right())
        .map(|x| buf[(x, row)].symbol())
        .collect()
}

fn interior_slice(buf: &Buffer, rect: Rect, row: u16) -> String {
    (rect.x + 1..rect.right() - 1)
        .map(|x| buf[(x, row)].symbol())
        .collect()
}

fn is_pua(ch: char) -> bool {
    (FONT_FIRST..=FONT_LAST).contains(&u32::from(ch))
}

fn is_pip_char(ch: char, mode: GlyphMode) -> bool {
    match mode {
        GlyphMode::Font => is_pua(ch) || matches!(ch, '█' | '▀' | '▄'),
        GlyphMode::Unicode => matches!(ch, '█' | '▀' | '▄'),
        GlyphMode::Ascii => matches!(ch, '#' | '/' | '\\' | '^' | 'v' | '<' | '>'),
    }
}

fn visible_rows_of(
    game: &dyn Game,
    pile: PileRef,
    cards: &[Rect],
    index: usize,
    size: CardSize,
) -> u16 {
    let rect = cards[index];
    if let Some(next) = cards.get(index + 1)
        && next.x == rect.x
        && next.y > rect.y
        && next.y < rect.bottom()
    {
        return next.y - rect.y;
    }
    if game.is_covered(pile) {
        return size.pyramid_step();
    }
    rect.height
}

fn assert_card_header(
    buf: &Buffer,
    rect: Rect,
    visible: u16,
    card: Card,
    glyphs: &Glyphs,
    what: &str,
) {
    let corner = label(card, glyphs);
    let top = row_slice(buf, rect, rect.y);
    let after_corner: String = top.chars().skip(1).collect();
    assert!(
        after_corner.starts_with(&corner),
        "{what}: corner {corner:?} missing in {top:?}"
    );
    if visible >= 2 && rect.width >= 9 && rect.height < 9 {
        let header = row_slice(buf, rect, rect.y + 1);
        let rank = rank_in(card.rank, glyphs);
        let suit = glyphs.suit(card.suit);
        assert!(
            header.contains(&rank) && header.contains(suit),
            "{what}: header {rank} {suit} missing in {header:?}"
        );
    }
}

/// A pip is partial when any pip symbol is visible in rows below the header while the whole pip
/// area (`pip_top..pip_bottom`) does not fit inside `visible` uncovered rows.
fn assert_no_partial_pip(
    buf: &Buffer,
    rect: Rect,
    visible: u16,
    size: CardSize,
    mode: GlyphMode,
    what: &str,
) {
    let (_, pip_bottom) = pip_bounds(size);
    if visible >= pip_bottom {
        return;
    }
    for rel_row in 2..visible.min(rect.height - 1) {
        let inside = interior_slice(buf, rect, rect.y + rel_row);
        assert!(
            !inside.chars().any(|ch| is_pip_char(ch, mode)),
            "{what}: partial pip on rel_row {rel_row} ({inside:?})"
        );
        assert!(
            inside.trim().is_empty(),
            "{what}: expected blank strip below header on rel_row {rel_row}, got {inside:?}"
        );
    }
}

fn assert_full_pip_present(
    buf: &Buffer,
    rect: Rect,
    size: CardSize,
    card: Card,
    glyphs: &Glyphs,
    what: &str,
) {
    let (pip_top, pip_bottom) = pip_bounds(size);
    let mode = glyphs.mode();
    let rows: Vec<String> = (pip_top..pip_bottom)
        .map(|rel| interior_slice(buf, rect, rect.y + rel))
        .collect();
    if size.width < 9 {
        let suit = glyphs.suit(card.suit);
        assert!(rows[0].contains(suit), "{what}: missing {suit} in {rows:?}");
        return;
    }
    if mode == GlyphMode::Font {
        assert!(
            rows.iter().any(|row| row.chars().any(is_pua)),
            "{what}: missing font glyph in {rows:?}"
        );
        return;
    }
    let non_empty = rows
        .iter()
        .filter(|row| row.chars().any(|ch| is_pip_char(ch, mode)))
        .count();
    assert_eq!(
        non_empty,
        usize::from(pip_bottom - pip_top),
        "{what}: {rows:?}"
    );
}

pub(super) fn check_board_buffer(
    buf: &Buffer,
    game: &dyn Game,
    piles: &[PileSnapshot],
    geometry: &BoardGeometry,
    (glyphs, mode): (&Glyphs, GlyphMode),
    what: &str,
) {
    let mut saw_covered = false;
    let mut saw_uncovered = false;
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        for (index, table_card) in snapshot.cards.iter().enumerate() {
            let rect = placed.cards[index];
            let stacked = placed.cards.get(index + 1) == Some(&rect);
            let fanned = placed
                .cards
                .get(index + 1)
                .is_some_and(|next| next.y == rect.y && next.x > rect.x);
            if !table_card.face_up || stacked || fanned {
                continue;
            }
            let visible = visible_rows_of(game, snapshot.pile, &placed.cards, index, geometry.card);
            let card_what = format!("{what} {:?}[{index}] vis={visible}", snapshot.pile);
            assert_card_header(buf, rect, visible, table_card.card, glyphs, &card_what);
            if visible < rect.height {
                saw_covered = true;
                assert_no_partial_pip(buf, rect, visible, geometry.card, mode, &card_what);
            } else {
                saw_uncovered = true;
                assert_full_pip_present(
                    buf,
                    rect,
                    geometry.card,
                    table_card.card,
                    glyphs,
                    &card_what,
                );
            }
        }
    }
    assert!(saw_covered && saw_uncovered, "{what}");
}

pub(super) fn render_piles(
    game: &dyn Game,
    piles: &[PileSnapshot],
    geometry: &BoardGeometry,
    area: Rect,
    mode: GlyphMode,
) -> (Buffer, Glyphs) {
    let settings = settings_for(mode, CardSizeSetting::of(geometry.card));
    let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
    let glyphs = Glyphs::styled(mode, &settings.front_style, &settings.back_style);
    let blocked: Vec<PileRef> = piles
        .iter()
        .map(|s| s.pile)
        .filter(|&pile| game.is_covered(pile))
        .collect();
    let marks = Marks {
        blocked,
        ..Marks::default()
    };
    let ctx = ViewCtx {
        theme: &theme,
        glyphs: &glyphs,
        settings: &settings,
        caps: &TRUE_COLOR,
        audio_status: AudioStatus::NoDevice,
        sound_styles: &[],
        saved: None,
        pointer: None,
        now: Instant::now(),
        font: crate::app::FontFacts::default(),
    };
    let mut buf = Buffer::empty(area);
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        draw_pile(&mut buf, snapshot, placed, &marks, false, &ctx);
    }
    (buf, glyphs)
}

#[test]
fn every_card_size_and_glyph_mode_in_the_three_games_has_no_partial_pips() {
    let area = Rect::new(0, 0, 160, 56);
    for variant in THREE_GAMES {
        let game = deal(variant, 7);
        let piles = game.piles();
        for size in SIZES {
            let geometry = layout::compute_within(variant, &piles, area, Some(size));
            assert_eq!(geometry.card, size, "{variant:?}");
            for mode in ALL_MODES {
                let (buf, glyphs) = render_piles(&*game, &piles, &geometry, area, mode);
                let what = format!("{variant:?} {size:?} {mode:?}");
                check_board_buffer(&buf, &*game, &piles, &geometry, (&glyphs, mode), &what);
            }
        }
    }
}
