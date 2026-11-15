//! Clipped cell painting shared by the parts of the card widget.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;

pub(super) struct Painter<'a> {
    pub(super) buf: &'a mut Buffer,
    pub(super) clip: Rect,
}

impl Painter<'_> {
    pub(super) fn put(&mut self, x: u16, y: u16, symbol: &str, style: Style) {
        if !self.clip.contains(Position::new(x, y)) {
            return;
        }
        if let Some(cell) = self.buf.cell_mut((x, y)) {
            cell.set_symbol(symbol).set_style(style);
        }
    }

    pub(super) fn text(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut at = x;
        for symbol in text.chars() {
            self.put(at, y, symbol.encode_utf8(&mut [0; 4]), style);
            at = at.saturating_add(1);
        }
    }

    pub(super) fn fill(&mut self, area: Rect, symbol: &str, style: Style) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                self.put(x, y, symbol, style);
            }
        }
    }
}

pub(super) fn width_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

pub(super) fn centered(area: Rect, text: &str) -> u16 {
    area.x + area.width.saturating_sub(width_of(text)) / 2
}
