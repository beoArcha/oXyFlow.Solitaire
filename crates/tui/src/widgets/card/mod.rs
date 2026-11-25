//! The card widget: a front with rank and suit in the corners, the same rank and suit as bold
//! text in the centre above a suit pictogram (in `Font` mode a glyph of the oXySol font), a back,
//! and an empty slot. The suit symbols, court labels and the back tile come from the card styles
//! of the settings through `Glyphs` (P5-02).
//!
//! The rank and suit sit in the top border row, so a card overlapped by the next card of its
//! column still shows them. Focus, selection and hint change the border shape; focus, selection,
//! target and hint each add a marker symbol. A blocked card (covered by a card still on the table)
//! has a dotted border and a dimmed face. None of them depends on color. The Unicode borders are
//! WGL4.

use oxysol_core::{Card, Rank};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

mod art;
pub(crate) mod back;
mod face;
mod frame;
mod mark;
mod paint;
mod rows;
mod slot;
#[cfg(test)]
mod tests;

use self::face::paint_center;
use self::frame::{edges_for, paint_frame, paint_patterned_top, paint_target_corners};
pub use self::mark::{Hint, Mark};
use self::paint::{Painter, centered};
use self::rows::{badge_text, paint_bottom_row, paint_top_row};
pub use self::slot::Placeholder;
use self::slot::paint_slot;
use crate::glyphs::Glyphs;
use crate::i18n::t;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// What a card position shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Up(Card),
    Down,
    Slot(Placeholder),
}

/// The localized label of a rank: `A`, `2` … `10`, `J`, `Q`, `K`.
pub fn rank_text(rank: Rank) -> String {
    let key = match rank {
        Rank::Ace => "board.rank.ace",
        Rank::Two => "board.rank.two",
        Rank::Three => "board.rank.three",
        Rank::Four => "board.rank.four",
        Rank::Five => "board.rank.five",
        Rank::Six => "board.rank.six",
        Rank::Seven => "board.rank.seven",
        Rank::Eight => "board.rank.eight",
        Rank::Nine => "board.rank.nine",
        Rank::Ten => "board.rank.ten",
        Rank::Jack => "board.rank.jack",
        Rank::Queen => "board.rank.queen",
        Rank::King => "board.rank.king",
    };
    t(key).into_owned()
}

/// The label of a rank in a glyph set: the label the front style gives to ace and court ranks, else
/// the localized one.
pub fn rank_in(rank: Rank, glyphs: &Glyphs) -> String {
    glyphs
        .rank_label(rank)
        .map_or_else(|| rank_text(rank), str::to_owned)
}

/// The corner label of a card: rank and suit, for example `10♥` or `10H`.
pub fn label(card: Card, glyphs: &Glyphs) -> String {
    format!("{}{}", rank_in(card.rank, glyphs), glyphs.suit(card.suit))
}

struct Look {
    fill: Style,
    border: Style,
}

fn look(face: Face, mark: Mark, ctx: &ViewCtx<'_>) -> Look {
    let theme = ctx.theme.with_color_vision(ctx.settings.color_vision);
    let blocked = mark.blocked && matches!(face, Face::Up(_));
    let fill = match face {
        Face::Up(card) => {
            if ctx.settings.four_color_deck {
                theme.card_suit_style(ctx.settings.card_face, card.suit, true)
            } else {
                theme.card_face_style(ctx.settings.card_face, ctx.glyphs.suit_token(card.suit))
            }
        }
        Face::Down => back::style(ctx),
        Face::Slot(_) => theme.style(Token::Slot, Token::Table),
    };
    let border = match mark.token() {
        Some(Token::BorderFocus) => theme.style(Token::BorderFocus, Token::Surface),
        Some(token) => theme.style(token, Token::Table),
        None => match face {
            Face::Slot(_) => theme.style(Token::Slot, Token::Table),
            _ => fill,
        },
    };
    Look {
        fill: shaded(fill, blocked),
        border: shaded(border, blocked && mark.token().is_none()),
    }
}

/// Sets or clears the dim attribute of a style. A cell keeps the modifiers of what was drawn on it
/// before, so every card states its own shade and a card drawn over a covered one is not dimmed.
fn shaded(style: Style, dim: bool) -> Style {
    if dim {
        style.add_modifier(Modifier::DIM)
    } else {
        style.remove_modifier(Modifier::DIM)
    }
}

/// Draws one card position into `area`. `badge` is a count shown in the bottom corner (the size
/// of a stock). The widget draws the rows that fit `area`; a card overlapped by another keeps its
/// top rows.
pub fn draw(
    buf: &mut Buffer,
    area: Rect,
    face: Face,
    mark: Mark,
    badge: Option<usize>,
    ctx: &ViewCtx<'_>,
) {
    if area.width < 3 || area.height < 2 {
        return;
    }
    let look = look(face, mark, ctx);
    let mut painter = Painter { buf, clip: area };
    let interior = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    match face {
        Face::Up(_) => painter.fill(interior, " ", look.fill),
        Face::Down => painter.fill(interior, back::pattern(ctx.glyphs), look.fill),
        Face::Slot(_) => painter.fill(interior, " ", look.border),
    }
    paint_frame(&mut painter, area, edges_for(ctx.glyphs, mark), look.border);
    let theme = ctx.theme.with_color_vision(ctx.settings.color_vision);
    let target = shaded(theme.style(Token::Target, Token::Table), false);
    if mark.target {
        paint_target_corners(&mut painter, area, ctx.glyphs, target);
    }
    paint_top_row(&mut painter, area, (face, mark), ctx.glyphs, &look);
    paint_bottom_row(&mut painter, area, (face, badge), ctx.glyphs, &look);
    match face {
        Face::Up(card) => paint_center(&mut painter, area, card, ctx.glyphs, look.fill),
        Face::Slot(placeholder) => paint_slot(
            &mut painter,
            area,
            (placeholder, mark.target),
            ctx.glyphs,
            (theme.style(Token::Slot, Token::Table), target),
        ),
        Face::Down => {}
    }
}

/// Draws a face-down card of which only the top row shows, as a row of the back pattern between
/// the corners (`┌▒▒▒▒▒┐`, `+#####+`), so a covered card is not mistaken for an empty slot.
pub fn draw_back_row(buf: &mut Buffer, area: Rect, mark: Mark, ctx: &ViewCtx<'_>) {
    if area.width < 3 || area.height < 1 {
        return;
    }
    let top = Rect::new(area.x, area.y, area.width, 1);
    let look = look(Face::Down, mark, ctx);
    let mut painter = Painter { buf, clip: top };
    let edges = edges_for(ctx.glyphs, mark);
    let pattern = back::pattern(ctx.glyphs);
    paint_patterned_top(&mut painter, top, edges, pattern, look.border);
}

/// Draws a card of a fan that the next card covers from `visible` columns on: the whole card, then
/// the rows under its top row blank over the visible strip, so that only the border and the
/// corner label show and no piece of the centre or of the bottom border peeks out.
pub fn draw_fanned(
    buf: &mut Buffer,
    area: Rect,
    visible: u16,
    (face, mark): (Face, Mark),
    ctx: &ViewCtx<'_>,
) {
    draw(buf, area, face, mark, None, ctx);
    let look = look(face, mark, ctx);
    let inner = Rect::new(
        area.x + 1,
        area.y + 1,
        visible.saturating_sub(1),
        area.height.saturating_sub(2),
    );
    let mut painter = Painter { buf, clip: area };
    painter.fill(inner, " ", look.fill);
    let bottom = Rect::new(area.x, area.bottom().saturating_sub(1), visible, 1);
    painter.fill(bottom, " ", look.border);
}

/// Draws the one-row stand-in for `count` collapsed face-down cards.
pub fn draw_collapsed(buf: &mut Buffer, area: Rect, count: usize, ctx: &ViewCtx<'_>) {
    let style = back::style(ctx);
    let mut painter = Painter { buf, clip: area };
    let pattern = back::pattern(ctx.glyphs);
    painter.fill(area, pattern, style);
    let text = format!(" {} ", badge_text(count));
    painter.text(centered(area, &text), area.y, &text, style);
}
