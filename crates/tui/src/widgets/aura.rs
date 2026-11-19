//! The aura: a one-cell halo around the focused card or run, the picked-up run, and the piles that
//! accept it. The halo is drawn only on free table cells, never over a card, so it moves nothing
//! and covers nothing. Each halo has its own symbol, so the three read apart without color: a light
//! shade for the focus, a dark shade for the carried run, a dot for a target.

use oxysol_core::PileRef;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;

use super::pile::{CardPos, Marks};
use crate::layout::BoardGeometry;
use crate::screens::ViewCtx;
use crate::settings::AuraMode;
use crate::theme::Token;

/// The cells of the halo of `rect`: its one-cell ring.
pub fn ring(rect: Rect, clip: Rect) -> impl Iterator<Item = (u16, u16)> {
    let (left, top) = (rect.left().saturating_sub(1), rect.top().saturating_sub(1));
    let (right, bottom) = (
        rect.right().saturating_add(1),
        rect.bottom().saturating_add(1),
    );
    let outer = Rect::new(left, top, right - left, bottom - top).intersection(clip);
    (outer.top()..outer.bottom())
        .flat_map(move |y| (outer.left()..outer.right()).map(move |x| (x, y)))
        .filter(move |(x, y)| !rect.contains(Position::new(*x, *y)))
}

fn union(a: Rect, b: Rect) -> Rect {
    let left = a.left().min(b.left());
    let top = a.top().min(b.top());
    Rect::new(
        left,
        top,
        a.right().max(b.right()) - left,
        a.bottom().max(b.bottom()) - top,
    )
}

/// The rectangle of the run that starts at `from` and reaches the top of its pile; the slot of an
/// empty pile. `None` for a pile the geometry does not know.
pub fn run_rect(geometry: &BoardGeometry, from: CardPos) -> Option<Rect> {
    let placed = geometry.pile(from.pile)?;
    placed
        .cards
        .get(from.index..)
        .and_then(|run| run.iter().copied().reduce(union))
        .or(Some(placed.slot))
}

fn top_rect(geometry: &BoardGeometry, pile: PileRef) -> Option<Rect> {
    let placed = geometry.pile(pile)?;
    placed.cards.last().copied().or(Some(placed.slot))
}

fn occupied(geometry: &BoardGeometry, x: u16, y: u16) -> bool {
    let at = Position::new(x, y);
    geometry.piles.iter().any(|placed| {
        let drawn_slot = placed.cards.is_empty() && !matches!(placed.pile, PileRef::Pyramid { .. });
        placed.cards.iter().any(|rect| rect.contains(at))
            || (drawn_slot && placed.slot.contains(at))
    })
}

fn paint(
    buf: &mut Buffer,
    geometry: &BoardGeometry,
    clip: Rect,
    rect: Rect,
    (symbol, style): (&str, Style),
) {
    for (x, y) in ring(rect, clip) {
        if occupied(geometry, x, y) {
            continue;
        }
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_symbol(symbol).set_style(style);
        }
    }
}

/// Draws the halos of `marks` over the free cells of `clip`: targets first, then the focus, then
/// the carried run, so the stronger halo wins where two meet. In [`AuraMode::FocusOnly`] the
/// targets and the carried run get no halo; their `+` and `●` card marks still show.
pub fn draw(
    buf: &mut Buffer,
    clip: Rect,
    geometry: &BoardGeometry,
    marks: &Marks,
    ctx: &ViewCtx<'_>,
) {
    let theme = ctx.theme;
    let glyphs = ctx.glyphs;
    let on = |token| theme.style(token, Token::Table);
    let full = ctx.settings.aura == AuraMode::Full;
    if full {
        for rect in marks
            .targets
            .iter()
            .filter_map(|pile| top_rect(geometry, *pile))
        {
            paint(
                buf,
                geometry,
                clip,
                rect,
                (glyphs.aura_target, on(Token::Target)),
            );
        }
    }
    let focus = (glyphs.aura_focus, on(Token::AuraFocus));
    if let Some(rect) = marks.focus.and_then(|pos| run_rect(geometry, pos)) {
        paint(buf, geometry, clip, rect, focus);
    }
    let carry = (glyphs.aura_carry, on(Token::AuraCarry));
    if let Some(rect) = marks
        .selected
        .and_then(|pos| run_rect(geometry, pos))
        .filter(|_| full)
    {
        paint(buf, geometry, clip, rect, carry);
    }
}
