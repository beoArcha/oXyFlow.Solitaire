//! The arrows that frame the value of the focused settings row, and the cells they were drawn in, so
//! that a click on one changes the value back or forward.

use ratatui::layout::Rect;

use crate::screens::ViewCtx;

/// The arrows of the focused row: the row's index and the cell of each arrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arrows {
    pub row: usize,
    pub left: Rect,
    pub right: Rect,
}

/// The (left, right) glyphs: `◄` and `►` in Unicode mode, `<` and `>` in ASCII.
pub fn glyphs(ctx: &ViewCtx<'_>) -> (&'static str, &'static str) {
    (ctx.glyphs.focus_right, ctx.glyphs.focus_left)
}

/// Finds the cells of the two arrows in the text drawn for row `row` at `cell`. The left arrow is
/// the first left glyph; the right arrow is the last right glyph, because the focus marker in front
/// of the label can be the same glyph as the right arrow.
pub fn locate(row: usize, text: &str, cell: Rect, (left, right): (&str, &str)) -> Option<Arrows> {
    let at = |byte: usize| {
        let column = u16::try_from(text[..byte].chars().count()).ok()?;
        Some(Rect::new(cell.x.saturating_add(column), cell.y, 1, 1))
    };
    Some(Arrows {
        row,
        left: at(text.find(left)?)?,
        right: at(text.rfind(right)?)?,
    })
}
