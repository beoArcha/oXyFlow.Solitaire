//! The back of a face-down card. The back style (settings row Back style) chooses the tile that
//! fills the card. The Card back setting chooses the palette: `Theme` keeps the colors of the
//! style's tokens in the current theme, the other values take the same tokens from another theme.
//!
//! A back has no centre mark. The style's 2 × 2 mosaic would have to sit in the middle of a card
//! whose width and height are odd, so it could only sit half a cell off centre; the stock showed
//! those halves as stray bars.

use ratatui::style::Style;

use crate::glyphs::Glyphs;
use crate::screens::ViewCtx;
use crate::theme::Token;

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
