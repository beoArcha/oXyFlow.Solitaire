//! Mouse gestures on the board. Each gesture first puts the cursor on the card or pile under the
//! pointer, then performs the same [`Action`] the keyboard performs: a click picks up or drops, a
//! click on the stock draws, a double click quick-moves, a right click cancels the pick. A drag is
//! a press that picks a card, followed by a release over another pile. The hit test is
//! [`BoardGeometry::hit`](crate::layout::BoardGeometry::hit), so the pointer meets the board as it
//! is drawn.

use std::time::{Duration, Instant};

use oxysol_core::{CardId, PileRef, PileSnapshot};
use ratatui::crossterm::event::MouseEvent;
use ratatui::layout::Position;

use super::cursor::{Cursor, focusable};
use super::{BoardScreen, toggle_guide};
use crate::action::Action;
use crate::layout;
use crate::mouse::{left_press, left_release, right_press};
use crate::screens::{Effect, ViewCtx};
use crate::widgets::guide;

const DOUBLE_CLICK: Duration = Duration::from_millis(400);

/// Where the pointer is: a pile, the depth of the card in it, and the card's identity. An empty
/// slot has no card and depth `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Spot {
    pile: PileRef,
    depth: usize,
    card: Option<CardId>,
}

/// The last press that landed on a card, for double-click detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LastPress {
    card: CardId,
    at: Instant,
}

/// The pointer state of the board: the last press on a card, and the pile a drag started from.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Pointer {
    last: Option<LastPress>,
    drag_from: Option<PileRef>,
}

impl Pointer {
    fn is_double(&self, card: CardId, now: Instant) -> bool {
        self.last.is_some_and(|last| {
            last.card == card && now.saturating_duration_since(last.at) <= DOUBLE_CLICK
        })
    }
}

fn depth_of(snapshot: &PileSnapshot, index: usize) -> usize {
    match snapshot.pile {
        PileRef::Tableau(_) => snapshot.cards.len().saturating_sub(index),
        _ => 1,
    }
}

impl BoardScreen {
    /// Handles a mouse event on the board: a left press picks or drops, a release ends a drag, a
    /// right press cancels a pick. Other events, and presses that land on nothing, do nothing.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        if self.session.is_none() {
            return Effect::None;
        }
        self.guide_panel = ctx.settings.guide_panel;
        if let Some(cell) = left_press(event) {
            if self.on_guide_title(cell) {
                return toggle_guide(ctx);
            }
            if self.on_seed(cell, ctx.now) {
                return self.dispatch(Action::CopySeed);
            }
            return self.press(cell, ctx.now);
        }
        if let Some(cell) = left_release(event) {
            return self.release(cell);
        }
        if right_press(event).is_some() && self.pick.is_some() {
            return self.dispatch(Action::Back);
        }
        Effect::None
    }

    fn on_seed(&self, cell: (u16, u16), now: Instant) -> bool {
        let header = layout::chrome_with(self.screen, self.guide_panel).header;
        self.header(now).is_some_and(|text| {
            text.seed_cells(header)
                .contains(Position::new(cell.0, cell.1))
        })
    }

    fn on_guide_title(&self, cell: (u16, u16)) -> bool {
        layout::chrome_with(self.screen, self.guide_panel)
            .guide
            .map(guide::title_row)
            .is_some_and(|row| row.contains(Position::new(cell.0, cell.1)))
    }

    fn press(&mut self, cell: (u16, u16), now: Instant) -> Effect {
        let Some(spot) = self.spot_at(cell) else {
            self.pointer = Pointer::default();
            return Effect::None;
        };
        if spot.pile == PileRef::Stock {
            self.pointer = Pointer::default();
            self.point_at(spot);
            return self.dispatch(Action::Stock);
        }
        let doubled = spot
            .card
            .is_some_and(|card| self.pointer.is_double(card, now));
        self.pointer.last = spot.card.map(|card| LastPress { card, at: now });
        self.point_at(spot);
        if doubled {
            self.pointer = Pointer::default();
            return self.dispatch(Action::QuickMove);
        }
        let picking = self.pick.is_none();
        let effect = self.dispatch(Action::Select);
        self.pointer.drag_from = (picking && self.pick.is_some()).then_some(spot.pile);
        effect
    }

    fn release(&mut self, cell: (u16, u16)) -> Effect {
        let Some(source) = self.pointer.drag_from.take() else {
            return Effect::None;
        };
        let Some(spot) = self.spot_at(cell) else {
            return Effect::None;
        };
        if spot.pile == source || self.pick.is_none() {
            return Effect::None;
        }
        self.point_at(spot);
        self.dispatch(Action::Select)
    }

    fn point_at(&mut self, spot: Spot) {
        self.cursor = Cursor {
            pile: spot.pile,
            depth: spot.depth,
        };
    }

    fn spot_at(&self, cell: (u16, u16)) -> Option<Spot> {
        let piles = self.piles();
        let geometry = self.geometry(&piles)?;
        let hit = geometry.hit(cell.0, cell.1)?;
        let snapshot = piles.iter().find(|snapshot| snapshot.pile == hit.pile)?;
        match hit.card {
            Some(index) => Some(Spot {
                pile: hit.pile,
                depth: depth_of(snapshot, index),
                card: Some(snapshot.cards.get(index)?.card.id),
            }),
            None if focusable(snapshot) => Some(Spot {
                pile: hit.pile,
                depth: 1,
                card: None,
            }),
            None => None,
        }
    }
}
