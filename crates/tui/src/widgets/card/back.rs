//! The back of a face-down card. The back style (settings row Back style) chooses the pattern: the
//! tile that fills the card and the 2 × 2 mosaic in the middle of a large one. The Card back setting
//! chooses the palette: `Theme` keeps the colors of the style's tokens in the current theme, the
//! other values take the same tokens from another theme.

use ratatui::layout::Rect;
use ratatui::style::Style;

use super::paint::Painter;
use crate::glyphs::Glyphs;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// How many pattern symbols the settings row shows as its preview.
const PREVIEW_WIDTH: usize = 4;
/// The smallest card the mosaic is drawn on (the Large size, 11 × 9).
const MOSAIC_MIN: (u16, u16) = (11, 9);
/// The smallest character cell, in pixels, in which the 2 × 2 mosaic of the card font is drawn.
const MOSAIC_MIN_CELL: (u16, u16) = (10, 24);

/// The fill pattern symbol of the glyph set: the tile of its back style.
pub(crate) fn pattern(glyphs: &Glyphs) -> &'static str {
    glyphs.face_down
}

/// The tokens of the pattern and of the background of the glyph set's back style.
pub(crate) fn tokens(glyphs: &Glyphs) -> (Token, Token) {
    glyphs
        .back
        .map_or((Token::CardBackMark, Token::CardBack), |back| {
            (back.fg, back.bg)
        })
}

/// The fill style of a card back.
pub(super) fn style(ctx: &ViewCtx<'_>) -> Style {
    ctx.theme
        .card_back_style(ctx.settings.card_back, tokens(ctx.glyphs))
}

/// A short run of the back's pattern, shown after the name of the card back in the settings row.
pub(crate) fn preview(glyphs: &Glyphs) -> String {
    pattern(glyphs).repeat(PREVIEW_WIDTH)
}

/// Draws the 2 × 2 mosaic in the middle of a back of `area`, only when the card is Large or Huge
/// and the style has a mosaic for the glyph mode. In `Font` mode on a console that tells its cell
/// size the cell must also be at least 10 × 24 pixels. Elsewhere the single tile is used.
pub(super) fn paint_mosaic(
    painter: &mut Painter<'_>,
    area: Rect,
    glyphs: &Glyphs,
    (style, cell_px): (Style, Option<(u16, u16)>),
) {
    let Some(back) = glyphs.back else { return };
    if area.width < MOSAIC_MIN.0 || area.height < MOSAIC_MIN.1 {
        return;
    }
    let small_cell = cell_px.is_some_and(|(w, h)| w < MOSAIC_MIN_CELL.0 || h < MOSAIC_MIN_CELL.1);
    if glyphs.font && small_cell {
        return;
    }
    let x = area.x + area.width.saturating_sub(2) / 2;
    let y = area.y + area.height.saturating_sub(2) / 2;
    for (offset, row) in (0..).zip(back.mosaic_rows(glyphs.mode())) {
        painter.text(x, y + offset, &row, style);
    }
}
