//! The texture of the felt for the table feel: a sparse dot (`Felt`) or faint rules (`Lined`). It is
//! drawn right after the felt and before the cards, auras and text, and only on blank cells, so it
//! never sits under a card, a frame or a line of text.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::Theme;
use super::palette::swatch;
use super::table::mix;
use crate::settings::TableFeel;

/// The share of the text colour mixed into the felt for the texture: low contrast on purpose.
const TEXTURE_PERCENT: i32 = 12;

/// One dot in five cells of a `Felt` table.
const DOT_EVERY: u32 = 5;

/// A rule on every third row of a `Lined` table, starting at the second row.
const RULE_EVERY: u16 = 3;
const RULE_OFFSET: u16 = 1;

/// Draws the texture of `feel` on the blank cells of `area`. `Plain` draws nothing; `unicode` picks
/// the dot and the rule glyphs of the Unicode set, else their ASCII forms.
pub fn draw(buffer: &mut Buffer, area: Rect, theme: &Theme, feel: TableFeel, unicode: bool) {
    if feel == TableFeel::Plain {
        return;
    }
    let style = texture_style(theme);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let glyph = match feel {
                TableFeel::Felt if dot_at(x, y) => dot(unicode),
                TableFeel::Lined if rule_at(area, y) => rule(unicode),
                _ => continue,
            };
            let Some(cell) = buffer.cell_mut((x, y)) else {
                continue;
            };
            if cell.symbol() == " " {
                cell.set_symbol(glyph).set_style(style);
            }
        }
    }
}

/// The style of the texture: the felt behind it, and a dim mix of the table text as the glyph.
fn texture_style(theme: &Theme) -> Style {
    let felt = theme.palette.table;
    let [r, g, b] = mix(felt.rgb, theme.palette.table_text.rgb, TEXTURE_PERCENT);
    let ink = swatch(r, g, b, Color::DarkGray);
    Style::new()
        .fg(ink.resolve(theme.depth))
        .bg(felt.resolve(theme.depth))
}

fn dot_at(x: u16, y: u16) -> bool {
    (u32::from(x) + 2 * u32::from(y)) % DOT_EVERY == 0
}

fn rule_at(area: Rect, y: u16) -> bool {
    (y - area.top()) % RULE_EVERY == RULE_OFFSET
}

const fn dot(unicode: bool) -> &'static str {
    if unicode { "·" } else { "." }
}

const fn rule(unicode: bool) -> &'static str {
    if unicode { "─" } else { "-" }
}
