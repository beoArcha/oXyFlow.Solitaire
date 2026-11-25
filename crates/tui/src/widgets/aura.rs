//! The aura: a one-cell halo around the focused card or run, the picked-up run, and the piles that
//! accept it. The halo is drawn only on free table cells, never over a card, so it moves nothing
//! and covers nothing. Each halo has its own symbol, so the three read apart without color: a light
//! shade for the focus, a dark shade for the carried run, a dot for a target.

use oxysol_core::PileRef;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::pile::{CardPos, Marks};
use crate::glyphs::Glyphs;
use crate::layout::BoardGeometry;
use crate::screens::ViewCtx;
use crate::settings::{AuraMode, AuraStyle};
use crate::theme::Token;

/// Which part of its ring a cell is: indexes of a frame set (four corners, then the horizontal and
/// the vertical line).
type Part = usize;

const TOP_LEFT: Part = 0;
const TOP_RIGHT: Part = 1;
const BOTTOM_LEFT: Part = 2;
const BOTTOM_RIGHT: Part = 3;
const HORIZONTAL: Part = 4;
const VERTICAL: Part = 5;

const FOCUS_FRAME: [&str; 6] = ["┌", "┐", "└", "┘", "─", "│"];
const CARRY_FRAME: [&str; 6] = ["╔", "╗", "╚", "╝", "═", "║"];
const FOCUS_FRAME_ASCII: [&str; 6] = ["+", "+", "+", "+", "-", "|"];
const CARRY_FRAME_ASCII: [&str; 6] = ["#", "#", "#", "#", "=", "H"];

/// The three halos of the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Focus,
    Carry,
    Target,
}

impl Role {
    /// The token the halo is drawn in when the colour setting is the theme's own.
    pub const fn token(self) -> Token {
        match self {
            Role::Focus => Token::AuraFocus,
            Role::Carry => Token::AuraCarry,
            Role::Target => Token::Target,
        }
    }
}

/// The symbol a halo of `role` shows on the side of a card in `style`, for a sample; `None` when
/// the style draws no halo.
pub fn side_symbol(glyphs: &Glyphs, style: AuraStyle, role: Role) -> Option<&'static str> {
    let halo = match role {
        Role::Focus => glyphs.aura_focus,
        Role::Carry => glyphs.aura_carry,
        Role::Target => glyphs.aura_target,
    };
    let frame = match (role, glyphs.unicode) {
        (Role::Focus, true) => &FOCUS_FRAME,
        (Role::Focus, false) => &FOCUS_FRAME_ASCII,
        (Role::Carry, true) => &CARRY_FRAME,
        (Role::Carry, false) => &CARRY_FRAME_ASCII,
        (Role::Target, _) => return Some(halo),
    };
    match style {
        AuraStyle::Halo => Some(halo),
        AuraStyle::Frame => frame.get(VERTICAL).copied(),
        AuraStyle::MarksOnly => None,
    }
}

fn coordinate(value: u16, offset: i32) -> i32 {
    i32::from(value) + offset
}

/// The cells of the ring of `rect` that is `depth` cells away from it (1 is the cell next to it),
/// cut to `clip`, each with its part of the ring.
fn ring_at(rect: Rect, clip: Rect, depth: u16) -> impl Iterator<Item = (u16, u16, Part)> {
    let depth = i32::from(depth);
    let left = coordinate(rect.left(), -depth);
    let top = coordinate(rect.top(), -depth);
    let right = coordinate(rect.right(), depth - 1);
    let bottom = coordinate(rect.bottom(), depth - 1);
    let outer = Rect::new(
        u16::try_from(left.max(0)).unwrap_or(0),
        u16::try_from(top.max(0)).unwrap_or(0),
        u16::try_from(right - left.max(0) + 1).unwrap_or(0),
        u16::try_from(bottom - top.max(0) + 1).unwrap_or(0),
    )
    .intersection(clip);
    (outer.top()..outer.bottom()).flat_map(move |y| {
        (outer.left()..outer.right()).filter_map(move |x| {
            let (column, row) = (i32::from(x), i32::from(y));
            let part = match (column == left, column == right, row == top, row == bottom) {
                (true, _, true, _) => TOP_LEFT,
                (_, true, true, _) => TOP_RIGHT,
                (true, _, _, true) => BOTTOM_LEFT,
                (_, true, _, true) => BOTTOM_RIGHT,
                (_, _, true, _) | (_, _, _, true) => HORIZONTAL,
                (true, _, _, _) | (_, true, _, _) => VERTICAL,
                _ => return None,
            };
            Some((x, y, part))
        })
    })
}

/// The cells of the halo of `rect`: its one-cell ring.
pub fn ring(rect: Rect, clip: Rect) -> impl Iterator<Item = (u16, u16)> {
    ring_at(rect, clip, 1).map(|(x, y, _)| (x, y))
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

/// What one halo is drawn with: its symbol, the frame set that replaces the symbol in the frame
/// style, and its colour.
struct Look {
    symbol: &'static str,
    frame: Option<&'static [&'static str; 6]>,
    style: Style,
}

fn paint(
    buf: &mut Buffer,
    geometry: &BoardGeometry,
    clip: Rect,
    rect: Rect,
    (look, rings): (&Look, u16),
) {
    for depth in 1..=rings {
        for (x, y, part) in ring_at(rect, clip, depth) {
            if occupied(geometry, x, y) {
                continue;
            }
            let symbol = look
                .frame
                .and_then(|frame| frame.get(part).copied())
                .unwrap_or(look.symbol);
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(symbol).set_style(look.style);
            }
        }
    }
}

/// Draws the halos of `marks` over the free cells of `clip`: targets first, then the focus, then
/// the carried run, so the stronger halo wins where two meet. In [`AuraMode::FocusOnly`] the
/// targets and the carried run get no halo; their `+` and `●` card marks still show. The style,
/// size and colour come from the settings; [`AuraStyle::MarksOnly`] draws no halo at all.
pub fn draw(
    buf: &mut Buffer,
    clip: Rect,
    geometry: &BoardGeometry,
    marks: &Marks,
    ctx: &ViewCtx<'_>,
) {
    let settings = ctx.settings;
    if settings.aura_style == AuraStyle::MarksOnly {
        return;
    }
    let theme = ctx.theme;
    let glyphs = ctx.glyphs;
    let rings = settings.aura_size.rings();
    let framed = settings.aura_style == AuraStyle::Frame;
    let tint = settings.aura_colour.token();
    let look = |symbol, token, frame: Option<&'static [&'static str; 6]>| {
        let base = theme.style(tint.unwrap_or(token), Token::Table);
        let style = if settings.low_vision {
            base.add_modifier(Modifier::BOLD)
        } else {
            base
        };
        Look {
            symbol,
            frame: frame.filter(|_| framed),
            style,
        }
    };
    let (focus_frame, carry_frame) = if glyphs.unicode {
        (&FOCUS_FRAME, &CARRY_FRAME)
    } else {
        (&FOCUS_FRAME_ASCII, &CARRY_FRAME_ASCII)
    };
    if settings.aura == AuraMode::Full {
        let target = look(glyphs.aura_target, Token::Target, None);
        for rect in marks
            .targets
            .iter()
            .filter_map(|pile| top_rect(geometry, *pile))
        {
            paint(buf, geometry, clip, rect, (&target, rings));
        }
    }
    let focus = look(glyphs.aura_focus, Token::AuraFocus, Some(focus_frame));
    if let Some(rect) = marks.focus.and_then(|pos| run_rect(geometry, pos)) {
        paint(buf, geometry, clip, rect, (&focus, rings));
    }
    let carry = look(glyphs.aura_carry, Token::AuraCarry, Some(carry_frame));
    if let Some(rect) = marks
        .selected
        .and_then(|pos| run_rect(geometry, pos))
        .filter(|_| settings.aura == AuraMode::Full)
    {
        paint(buf, geometry, clip, rect, (&carry, rings));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyphs::is_wgl4;

    #[test]
    fn the_frame_symbols_are_wgl4_or_ascii_and_differ_between_the_roles() {
        for frame in [FOCUS_FRAME, CARRY_FRAME] {
            for symbol in frame {
                assert!(symbol.chars().all(is_wgl4), "{symbol}");
            }
        }
        for frame in [FOCUS_FRAME_ASCII, CARRY_FRAME_ASCII] {
            assert!(frame.iter().all(|symbol| symbol.is_ascii()));
        }
        assert_ne!(FOCUS_FRAME, CARRY_FRAME);
        assert_ne!(FOCUS_FRAME_ASCII, CARRY_FRAME_ASCII);
    }

    #[test]
    fn a_ring_has_corners_and_sides_and_a_thick_one_lies_one_cell_further_out() {
        let rect = Rect::new(5, 5, 3, 2);
        let clip = Rect::new(0, 0, 40, 20);
        let parts: Vec<Part> = ring_at(rect, clip, 1).map(|(_, _, part)| part).collect();
        for corner in [TOP_LEFT, TOP_RIGHT, BOTTOM_LEFT, BOTTOM_RIGHT] {
            assert_eq!(parts.iter().filter(|part| **part == corner).count(), 1);
        }
        assert_eq!(parts.len(), 2 * (3 + 2) + 4);
        let outer: Vec<(u16, u16, Part)> = ring_at(rect, clip, 2).collect();
        assert!(
            outer
                .iter()
                .all(|(x, y, _)| !rect.contains(Position::new(*x, *y)))
        );
        assert!(outer.contains(&(3, 3, TOP_LEFT)));
        assert!(ring_at(rect, Rect::new(0, 0, 4, 4), 2).all(|(x, y, _)| x < 4 && y < 4));
    }

    #[test]
    fn a_ring_at_the_screen_edge_is_cut_not_wrapped() {
        let rect = Rect::new(0, 0, 3, 2);
        let clip = Rect::new(0, 0, 40, 20);
        assert!(ring_at(rect, clip, 1).all(|(x, y, _)| x <= 3 && y <= 2));
        assert_eq!(ring_at(rect, clip, 1).count(), 3 + 2 + 1);
    }
}
