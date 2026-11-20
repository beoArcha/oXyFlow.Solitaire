//! The back of a face-down card: style and pattern selection according to [`CardBack`].

use ratatui::style::Style;

use crate::glyphs::Glyphs;
use crate::screens::ViewCtx;
use crate::settings::CardBack;

/// Returns the fill pattern symbol for the chosen card back and glyph mode.
pub(super) fn pattern(back: CardBack, glyphs: &Glyphs) -> &'static str {
    match back {
        CardBack::Theme => glyphs.face_down,
        CardBack::Monokai => {
            if glyphs.unicode {
                "░"
            } else {
                "%"
            }
        }
        CardBack::Dracula => {
            if glyphs.unicode {
                "┼"
            } else {
                "+"
            }
        }
    }
}

/// The fill style of a card back.
pub(super) fn style(back: CardBack, ctx: &ViewCtx<'_>) -> Style {
    ctx.theme.card_back_style(back)
}
