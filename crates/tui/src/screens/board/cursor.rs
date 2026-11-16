//! The cursor: a pile and a depth inside it, and movement between piles by geometry.

use oxysol_core::{PileRef, PileSnapshot};
use ratatui::layout::Rect;

use crate::action::Action;
use crate::layout::BoardGeometry;
use crate::screens::focus::step;

/// A direction of cursor movement on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub(super) const fn from_action(action: Action) -> Option<Self> {
        match action {
            Action::Up => Some(Dir::Up),
            Action::Down => Some(Dir::Down),
            Action::Left => Some(Dir::Left),
            Action::Right => Some(Dir::Right),
            _ => None,
        }
    }
}

/// The pile under the cursor and how many cards from its top the focus reaches (`1` is the top
/// card). Only a tableau column has a depth beyond `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Cursor {
    pub pile: PileRef,
    pub depth: usize,
}

/// Whether the cursor may rest on the pile: every pile except an empty Pyramid position.
pub(super) fn focusable(snapshot: &PileSnapshot) -> bool {
    !snapshot.cards.is_empty() || !matches!(snapshot.pile, PileRef::Pyramid { .. })
}

/// How deep the focus may reach into the pile: the face-up cards of a column, else one.
pub(super) fn max_depth(snapshot: &PileSnapshot) -> usize {
    if !matches!(snapshot.pile, PileRef::Tableau(_)) {
        return 1;
    }
    let face_up = snapshot
        .cards
        .iter()
        .rev()
        .take_while(|card| card.face_up)
        .count();
    face_up.max(1)
}

impl Cursor {
    /// Where the cursor starts: the first column, the bottom-left of a pyramid, else the first pile.
    pub(super) fn initial(piles: &[PileSnapshot]) -> Self {
        let preferred = [PileRef::Tableau(0), PileRef::Pyramid { row: 6, index: 0 }];
        let pile = preferred
            .into_iter()
            .find(|wanted| piles.iter().any(|snapshot| snapshot.pile == *wanted))
            .or_else(|| piles.first().map(|snapshot| snapshot.pile))
            .unwrap_or(PileRef::Stock);
        Self { pile, depth: 1 }
    }

    /// Deepens the focus by one card; `false` at the limit.
    pub(super) fn deeper(&mut self, snapshot: &PileSnapshot) -> bool {
        let deeper = self.depth < max_depth(snapshot);
        if deeper {
            self.depth += 1;
        }
        deeper
    }

    /// Lifts the focus by one card; `false` at the top card.
    pub(super) fn shallower(&mut self) -> bool {
        let shallower = self.depth > 1;
        if shallower {
            self.depth -= 1;
        }
        shallower
    }
}

fn center(rect: Rect) -> (i32, i32) {
    (
        2 * i32::from(rect.x) + i32::from(rect.width),
        2 * i32::from(rect.y) + i32::from(rect.height),
    )
}

const ROW_WEIGHT: i32 = 2;
const OFF_AXIS_WEIGHT: i32 = 4;

fn cost(from: (i32, i32), to: (i32, i32), dir: Dir) -> Option<i32> {
    let (dx, dy) = (to.0 - from.0, (to.1 - from.1) * ROW_WEIGHT);
    let (along, across) = match dir {
        Dir::Left => (-dx, dy),
        Dir::Right => (dx, dy),
        Dir::Up => (-dy, dx),
        Dir::Down => (dy, dx),
    };
    (along > 0).then(|| along + OFF_AXIS_WEIGHT * across.abs())
}

fn slot_of(geometry: &BoardGeometry, pile: PileRef) -> Option<Rect> {
    geometry
        .pile(pile)
        .map(|placed| placed.slot)
        .filter(|slot| slot.width > 0 && slot.height > 0)
}

/// The focusable pile nearest to `from` strictly in direction `dir`. A card row counts twice a
/// column, and a pile off the axis is penalized, so the pile straight across wins over a closer
/// one at an angle.
pub(super) fn nearest(
    piles: &[PileSnapshot],
    geometry: &BoardGeometry,
    from: PileRef,
    dir: Dir,
) -> Option<PileRef> {
    let origin = center(slot_of(geometry, from)?);
    piles
        .iter()
        .filter(|snapshot| snapshot.pile != from && focusable(snapshot))
        .filter_map(|snapshot| {
            let slot = slot_of(geometry, snapshot.pile)?;
            Some((snapshot.pile, cost(origin, center(slot), dir)?))
        })
        .min_by_key(|(_, cost)| *cost)
        .map(|(pile, _)| pile)
}

/// Reading order of the piles: stock, waste, foundations, cells, tableau columns, then pyramid
/// positions row by row.
fn reading_key(pile: PileRef) -> (u8, u8, u8) {
    match pile {
        PileRef::Stock => (0, 0, 0),
        PileRef::Waste => (1, 0, 0),
        PileRef::Foundation(index) => (2, index, 0),
        PileRef::Cell(index) => (3, index, 0),
        PileRef::Tableau(index) => (4, index, 0),
        PileRef::Pyramid { row, index } => (5, row, index),
    }
}

/// The focusable pile after `from` in reading order (`forward`) or before it, wrapping at both
/// ends. A cursor on a pile that is not focusable moves to the first or the last one.
pub(super) fn cycle(piles: &[PileSnapshot], from: PileRef, forward: bool) -> Option<PileRef> {
    let mut order: Vec<PileRef> = piles
        .iter()
        .filter(|snapshot| focusable(snapshot))
        .map(|snapshot| snapshot.pile)
        .collect();
    order.sort_by_key(|pile| reading_key(*pile));
    let next = match order.iter().position(|pile| *pile == from) {
        Some(at) => step(at, order.len(), forward),
        None if forward => 0,
        None => order.len().saturating_sub(1),
    };
    order.get(next).copied()
}

/// The focusable pile closest to `from` in any direction; used when the cursor's own pile
/// disappears (a pyramid position that was removed).
pub(super) fn closest(
    piles: &[PileSnapshot],
    geometry: &BoardGeometry,
    from: PileRef,
) -> Option<PileRef> {
    let origin = slot_of(geometry, from).map(center);
    piles
        .iter()
        .filter(|snapshot| focusable(snapshot))
        .filter_map(|snapshot| {
            let (x, y) = center(slot_of(geometry, snapshot.pile)?);
            let distance =
                origin.map_or(0, |(ox, oy)| (x - ox).abs() + (y - oy).abs() * ROW_WEIGHT);
            Some((snapshot.pile, distance))
        })
        .min_by_key(|(_, distance)| *distance)
        .map(|(pile, _)| pile)
}
