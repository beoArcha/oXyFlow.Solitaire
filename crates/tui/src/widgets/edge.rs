//! The edge of the playing field: a one-cell frame in the `Border` token around the felt, drawn on
//! the cells the board geometry leaves free. It gives the field an outline that does not depend on
//! the felt color, so in High Contrast, where the felt and the screen are both black, the field
//! still has an edge. The ASCII glyph mode draws it with `+`, `-` and `|`.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Widget};

use super::guide::ASCII_BORDER;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// Draws the frame on the outer ring of `field`; the cells inside it are left as they are.
pub fn draw(buf: &mut Buffer, field: Rect, ctx: &ViewCtx<'_>) {
    let set = if ctx.glyphs.unicode {
        border::PLAIN
    } else {
        ASCII_BORDER
    };
    Block::bordered()
        .border_set(set)
        .border_style(ctx.theme.style(Token::Border, Token::Background))
        .render(field, buf);
}
