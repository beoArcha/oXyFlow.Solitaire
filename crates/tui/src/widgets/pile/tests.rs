//! Tests of the pile drawing: on a fresh Pyramid deal the covered cards are drawn in the blocked
//! style, the cards of the bottom row in the playable one.

use std::time::Instant;

use oxysol_core::{PileRef, PyramidRules, Variant, deal};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::{Marks, draw};
use crate::audio::AudioStatus;
use crate::glyphs::Glyphs;
use crate::layout;
use crate::screens::ViewCtx;
use crate::settings::{GlyphMode, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};

const AREA: Rect = Rect::new(0, 0, 120, 40);
const SEED: u64 = 7;
const PYRAMID_CARDS: usize = 28;
const BOTTOM_ROW: usize = 7;

struct Fixture {
    theme: Theme,
    glyphs: Glyphs,
    settings: Settings,
}

impl Fixture {
    fn new(mode: GlyphMode) -> Self {
        Self {
            theme: Theme::new(ThemeId::Dark, TRUE_COLOR.color),
            glyphs: Glyphs::new(mode),
            settings: Settings::default(),
        }
    }

    fn ctx(&self) -> ViewCtx<'_> {
        ViewCtx {
            theme: &self.theme,
            glyphs: &self.glyphs,
            settings: &self.settings,
            caps: &TRUE_COLOR,
            audio_status: AudioStatus::NoDevice,
            sound_styles: &[],
            saved: None,
            pointer: None,
            now: Instant::now(),
            font: crate::app::FontFacts::default(),
        }
    }
}

fn is_pyramid(pile: PileRef) -> bool {
    matches!(pile, PileRef::Pyramid { .. })
}

/// The left edge of a card, on its second row: the one cell of a card that the card below does not
/// cover, whatever the card size.
fn edge_cell(buf: &Buffer, rect: Rect) -> String {
    buf[(rect.x, rect.y + 1)].symbol().to_owned()
}

#[test]
fn a_fresh_pyramid_deal_draws_the_covered_cards_in_the_blocked_style() {
    for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
        let (playable_edge, blocked_edge) = if mode == GlyphMode::Ascii {
            ("|", ":")
        } else {
            ("│", "·")
        };
        let fixture = Fixture::new(mode);
        let ctx = fixture.ctx();
        let variant = Variant::Pyramid(PyramidRules::Relaxed);
        let game = deal(variant, SEED);
        let piles = game.piles();
        let geometry = layout::compute(variant, &piles, AREA);
        let blocked: Vec<PileRef> = piles
            .iter()
            .map(|snapshot| snapshot.pile)
            .filter(|&pile| game.is_covered(pile))
            .collect();
        let marks = Marks {
            blocked: blocked.clone(),
            ..Marks::default()
        };
        let mut buf = Buffer::empty(AREA);
        for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
            draw(&mut buf, snapshot, placed, &marks, false, &ctx);
        }
        let mut playable = 0;
        let mut dimmed = 0;
        for snapshot in piles.iter().filter(|s| is_pyramid(s.pile)) {
            let Some(rect) = geometry
                .pile(snapshot.pile)
                .and_then(|placed| placed.cards.first().copied())
            else {
                unreachable!("every pyramid position has a rectangle")
            };
            let is_blocked = blocked.contains(&snapshot.pile);
            let expected = if is_blocked {
                blocked_edge
            } else {
                playable_edge
            };
            assert_eq!(
                edge_cell(&buf, rect),
                expected,
                "{mode:?} {:?}",
                snapshot.pile
            );
            let dim = buf[(rect.x + 1, rect.y + 1)]
                .modifier
                .contains(Modifier::DIM);
            assert_eq!(dim, is_blocked, "{mode:?} {:?}", snapshot.pile);
            if is_blocked {
                dimmed += 1;
            } else {
                playable += 1;
            }
        }
        assert_eq!(playable, BOTTOM_ROW, "{mode:?}");
        assert_eq!(dimmed, PYRAMID_CARDS - BOTTOM_ROW, "{mode:?}");
    }
}
