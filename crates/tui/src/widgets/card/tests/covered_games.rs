//! Edge-case tests for Pyramid single-child coverage and other column games.

use oxysol_core::{DrawCount, Move, PileRef, PileSnapshot, PyramidRules, SuitCount, Variant, deal};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use super::covered_board::{
    ALL_MODES, THREE_GAMES, check_board_buffer, render_piles, settings_for,
};
use crate::glyphs::Glyphs;
use crate::layout::{self, BoardGeometry, SIZES, chrome_with};
use crate::screens::Effect;
use crate::settings::{CardSizeSetting, GlyphMode};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

fn check_harness_screen(
    variant: Variant,
    screen: (u16, u16),
    mode: GlyphMode,
    card_size: CardSizeSetting,
) {
    let caps = if mode == GlyphMode::Ascii {
        PLAIN
    } else {
        TRUE_COLOR
    };
    let settings = settings_for(mode, card_size);
    let storage = MemoryStorage {
        settings: Some(settings.clone()),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, caps, screen.0, screen.1);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(7),
        undo_allowed: true,
    });
    h.lines();
    let Some(session) = h.app.board().session() else {
        unreachable!("a game is running")
    };
    let body = chrome_with(Rect::new(0, 0, screen.0, screen.1), true, false).body;
    let piles = session.game().piles();
    let geometry = layout::compute_within(variant, &piles, body, card_size.size());
    let glyphs = Glyphs::styled(mode, &settings.front_style, &settings.back_style);
    let what = format!("{variant:?} {screen:?} {mode:?} {card_size:?}");
    check_board_buffer(
        h.terminal.backend().buffer(),
        session.game(),
        &piles,
        &geometry,
        (&glyphs, mode),
        &what,
    );
}

#[test]
fn screen_renders_at_hundred_by_thirty_and_one_thirty_by_forty_have_no_partial_pips() {
    let card_sizes = [
        CardSizeSetting::Small,
        CardSizeSetting::Medium,
        CardSizeSetting::Large,
        CardSizeSetting::Huge,
    ];
    for variant in THREE_GAMES {
        for screen in [(100, 30), (130, 40)] {
            for (mode, card_size) in ALL_MODES
                .into_iter()
                .flat_map(|m| card_sizes.into_iter().map(move |s| (m, s)))
            {
                check_harness_screen(variant, screen, mode, card_size);
            }
        }
    }
}

fn check_pyramid_exposed_half(
    buf: &Buffer,
    piles: &[PileSnapshot],
    geometry: &BoardGeometry,
    (row, index): (u8, u8),
    mode: GlyphMode,
) {
    let pile = PileRef::Pyramid { row, index };
    let Some(rect) = geometry.pile(pile).and_then(|p| p.cards.first().copied()) else {
        return;
    };
    let children: Vec<Rect> = [index, index + 1]
        .into_iter()
        .filter(|&child| {
            piles.iter().any(|s| {
                s.pile
                    == PileRef::Pyramid {
                        row: row + 1,
                        index: child,
                    }
                    && !s.cards.is_empty()
            })
        })
        .filter_map(|child| {
            geometry
                .pile(PileRef::Pyramid {
                    row: row + 1,
                    index: child,
                })?
                .cards
                .first()
                .copied()
        })
        .collect();
    for rel_row in 2..rect.height - 1 {
        for x in rect.x + 1..rect.right() - 1 {
            let pos = Position::new(x, rect.y + rel_row);
            if children.iter().any(|child| child.contains(pos)) {
                continue;
            }
            let sym = buf[(pos.x, pos.y)].symbol();
            assert_eq!(sym, " ", "{pile:?} {mode:?} ({x},{rel_row}): {sym:?}");
        }
    }
}

#[test]
fn pyramid_card_with_one_child_removed_still_shows_no_partial_pip_on_exposed_half() {
    let area = Rect::new(0, 0, 160, 56);
    let variant = Variant::Pyramid(PyramidRules::Relaxed);
    let mut game = deal(variant, 1);
    let remove = game
        .legal_moves()
        .into_iter()
        .find(|mv| matches!(mv, Move::Remove { second: None, .. }));
    let Some(mv) = remove else {
        unreachable!("seed 1 has a bottom-row king")
    };
    assert_eq!(game.apply(mv), Ok(()));
    let piles = game.piles();
    for size in SIZES {
        let geometry = layout::compute_within(variant, &piles, area, Some(size));
        for mode in ALL_MODES {
            let (buf, _) = render_piles(&*game, &piles, &geometry, area, mode);
            for (row, index) in (0..6_u8).flat_map(|r| (0..=r).map(move |i| (r, i))) {
                if game.is_covered(PileRef::Pyramid { row, index }) {
                    check_pyramid_exposed_half(&buf, &piles, &geometry, (row, index), mode);
                }
            }
        }
    }
}

#[test]
fn other_column_games_never_show_partial_pips_on_covered_face_up_cards() {
    let area = Rect::new(0, 0, 160, 56);
    let others = [
        Variant::Klondike(DrawCount::Three),
        Variant::Spider(SuitCount::Two),
        Variant::Golf,
    ];
    for variant in others {
        let mut game = deal(variant, 7);
        for step in 0..10_usize {
            let moves = game.legal_moves();
            if let Some(mv) = moves.get(step % moves.len().max(1)).copied() {
                assert_eq!(game.apply(mv), Ok(()));
            }
        }
        let piles = game.piles();
        for size in SIZES {
            let geometry = layout::compute_within(variant, &piles, area, Some(size));
            for mode in ALL_MODES {
                let (buf, glyphs) = render_piles(&*game, &piles, &geometry, area, mode);
                check_board_buffer(
                    &buf,
                    &*game,
                    &piles,
                    &geometry,
                    (&glyphs, mode),
                    &format!("{variant:?} {size:?} {mode:?}"),
                );
            }
        }
    }
}
