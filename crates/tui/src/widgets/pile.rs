//! Drawing of one pile from its snapshot and geometry, with the highlights the controller asks for.

use oxysol_core::{PileRef, PileSnapshot};
use ratatui::buffer::Buffer;

use super::card::{self, Face, Hint, Mark, Placeholder};
use crate::layout::PileGeometry;
use crate::screens::ViewCtx;

/// A card of a pile; `index` counts from the bottom. For an empty pile it is `0` and means the slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardPos {
    pub pile: PileRef,
    pub index: usize,
}

/// What the foundations of the game collect; it decides the mark of an empty foundation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Completion {
    /// Built up from an ace (Klondike, FreeCell, Yukon).
    #[default]
    Ace,
    /// Built down from a king (Spider).
    King,
    /// The pairs that add up to 13 (Pyramid).
    Thirteen,
}

/// The highlights of the board. The controller (P3-03) fills it; an empty value highlights nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Marks {
    /// What the foundations of the game collect, from the session.
    pub completion: Completion,
    /// The card under the cursor.
    pub focus: Option<CardPos>,
    /// The first card of the picked run; it and every card above it count as selected.
    pub selected: Option<CardPos>,
    /// Piles that accept the picked run; the top card or the empty slot is highlighted.
    pub targets: Vec<PileRef>,
    /// Hinted cards, the moved ones first and the destination last; a hinted target pile is its
    /// top card, or `index: 0` when empty. A hint that names one pile has only a source.
    pub hints: Vec<CardPos>,
    /// Piles whose card a card still on the table covers (`Game::is_covered`), drawn in the blocked
    /// style.
    pub blocked: Vec<PileRef>,
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
            hint: self.hint_role(here),
            blocked: self.blocked.contains(&pile),
        }
    }

    fn hint_role(&self, here: CardPos) -> Option<Hint> {
        if !self.hints.contains(&here) {
            return None;
        }
        let last = self.hints.last()?;
        let moves_between_piles = self.hints.iter().any(|card| card.pile != last.pile);
        let target = moves_between_piles && *last == here;
        Some(if target { Hint::Target } else { Hint::Source })
    }
}

fn placeholder(pile: PileRef, recycle: bool, completion: Completion) -> Placeholder {
    match pile {
        PileRef::Foundation(_) => match completion {
            Completion::Ace => Placeholder::Ace,
            Completion::King => Placeholder::King,
            Completion::Thirteen => Placeholder::Thirteen,
        },
        PileRef::Stock if recycle => Placeholder::Recycle,
        PileRef::Stock => Placeholder::Spent,
        _ => Placeholder::None,
    }
}

/// How the next card of a pile lies over a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cover {
    Free,
    /// The next card starts this many columns to the right (a fanned waste).
    Beside(u16),
    /// The next card starts this many rows lower, covering the card from that row down.
    Below(u16),
}

fn cover(geometry: &PileGeometry, index: usize) -> Cover {
    let rect = geometry.cards[index];
    match geometry.cards.get(index + 1) {
        Some(next) if next.y == rect.y && next.x > rect.x => Cover::Beside(next.x - rect.x),
        Some(next) if next.x == rect.x && next.y > rect.y && next.y < rect.bottom() => {
            Cover::Below(next.y - rect.y)
        }
        _ => Cover::Free,
    }
}

/// Draws `snapshot` where `geometry` puts it. Cards covered by an identical rectangle are skipped.
/// An empty pyramid position draws nothing; any other empty pile draws its slot. `recycle` tells
/// whether an empty stock can be turned over again.
pub fn draw(
    buf: &mut Buffer,
    snapshot: &PileSnapshot,
    geometry: &PileGeometry,
    marks: &Marks,
    recycle: bool,
    ctx: &ViewCtx<'_>,
) {
    let pile = snapshot.pile;
    let len = snapshot.cards.len();
    if len == 0 {
        if !matches!(pile, PileRef::Pyramid { .. }) {
            let face = Face::Slot(placeholder(pile, recycle, marks.completion));
            card::draw(buf, geometry.slot, face, marks.mark(pile, 0, 0), None, ctx);
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
        let mark = marks.mark(pile, index, len);
        match cover(geometry, index) {
            Cover::Below(1) if !table_card.face_up => card::draw_back_row(buf, rect, mark, ctx),
            Cover::Below(visible) => card::draw_covered(buf, rect, visible, (face, mark), ctx),
            Cover::Beside(visible) => card::draw_fanned(buf, rect, visible, (face, mark), ctx),
            Cover::Free => {
                let shown_badge = badge.filter(|_| index + 1 == len);
                card::draw(buf, rect, face, mark, shown_badge, ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests;
