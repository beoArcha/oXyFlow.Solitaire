//! Drawing of one pile from its snapshot and geometry, with the highlights the controller asks for.

use oxysol_core::{PileRef, PileSnapshot};
use ratatui::buffer::Buffer;

use super::card::{self, Face, Mark};
use crate::layout::PileGeometry;
use crate::screens::ViewCtx;

/// A card of a pile; `index` counts from the bottom. For an empty pile it is `0` and means the slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardPos {
    pub pile: PileRef,
    pub index: usize,
}

/// The highlights of the board. The controller (P3-03) fills it; an empty value highlights nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Marks {
    /// The card under the cursor.
    pub focus: Option<CardPos>,
    /// The first card of the picked run; it and every card above it count as selected.
    pub selected: Option<CardPos>,
    /// Piles that accept the picked run; the top card or the empty slot is highlighted.
    pub targets: Vec<PileRef>,
    /// Hinted cards; a hinted target pile is its top card, or `index: 0` when empty.
    pub hints: Vec<CardPos>,
}

impl Marks {
    fn mark(&self, pile: PileRef, index: usize, len: usize) -> Mark {
        let here = CardPos { pile, index };
        let top = index + 1 >= len;
        Mark {
            focus: self.focus == Some(here),
            selected: self
                .selected
                .is_some_and(|picked| picked.pile == pile && index >= picked.index),
            target: top && self.targets.contains(&pile),
            hint: self.hints.contains(&here),
        }
    }
}

/// Draws `snapshot` where `geometry` puts it. Cards covered by an identical rectangle are skipped.
/// An empty pyramid position draws nothing; any other empty pile draws its slot.
pub fn draw(
    buf: &mut Buffer,
    snapshot: &PileSnapshot,
    geometry: &PileGeometry,
    marks: &Marks,
    ctx: &ViewCtx<'_>,
) {
    let pile = snapshot.pile;
    let len = snapshot.cards.len();
    if len == 0 {
        if !matches!(pile, PileRef::Pyramid { .. }) {
            card::draw(
                buf,
                geometry.slot,
                Face::Slot,
                marks.mark(pile, 0, 0),
                None,
                ctx,
            );
        }
        return;
    }
    if let Some(first) = geometry.cards.first().filter(|_| geometry.collapsed > 0) {
        card::draw_collapsed(buf, *first, geometry.collapsed, ctx);
    }
    let badge = (pile == PileRef::Stock).then_some(len);
    for (index, table_card) in snapshot.cards.iter().enumerate().skip(geometry.collapsed) {
        let rect = geometry.cards[index];
        if geometry.cards.get(index + 1) == Some(&rect) {
            continue;
        }
        let face = if table_card.face_up {
            Face::Up(table_card.card)
        } else {
            Face::Down
        };
        let shown_badge = badge.filter(|_| index + 1 == len);
        card::draw(
            buf,
            rect,
            face,
            marks.mark(pile, index, len),
            shown_badge,
            ctx,
        );
    }
}
