//! The `Legend` section of the guide panel: the four marks of the board, each with the symbol it
//! has in the current glyph set, packed into rows of the panel's width.

use ratatui::text::Line;

use super::available_mark;
use crate::i18n::t;
use crate::screens::ViewCtx;
use crate::theme::Token;

const GAP: &str = "  ";

fn items(ctx: &ViewCtx<'_>) -> [String; 5] {
    let glyphs = ctx.glyphs;
    [
        ("board.guide.legend_focus", glyphs.focus_left),
        ("board.guide.legend_available", available_mark(ctx)),
        ("board.guide.legend_picked", glyphs.selected),
        ("board.guide.legend_target", glyphs.target),
        ("board.guide.legend_hint", glyphs.hint_source),
    ]
    .map(|(key, symbol)| format!("{symbol} {}", t(key)))
}

/// The lines of the `Legend` section below its heading.
pub(super) fn lines(width: usize, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
    let style = ctx.theme.style(Token::Text, Token::Surface);
    let mut rows: Vec<String> = Vec::new();
    for item in items(ctx) {
        match rows.last_mut() {
            Some(row) if row.chars().count() + GAP.len() + item.chars().count() <= width => {
                row.push_str(GAP);
                row.push_str(&item);
            }
            _ => rows.push(item),
        }
    }
    rows.into_iter()
        .map(|row| Line::styled(row, style))
        .collect()
}
