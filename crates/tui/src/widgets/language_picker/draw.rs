//! Drawing of the language selector, its tooltip and its list.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use super::{LanguagePicker, Spots, index_of};
use crate::caps::ColorDepth;
use crate::i18n::Locale;
use crate::i18n::flags::{PIXELS, Rgb, flag};
use crate::mouse::contains;
use crate::screens::ViewCtx;
use crate::screens::focus::marker;
use crate::theme::Token;
use crate::theme::palette::swatch;
use crate::widgets::guide::ASCII_BORDER;

const FLAG_CELLS: u16 = 4;
const BODY_WITH_FLAG: (u16, u16) = (10, 2);
const BODY_WITH_CODE: (u16, u16) = (8, 1);

fn flags_shown(ctx: &ViewCtx<'_>) -> bool {
    ctx.glyphs.unicode && ctx.settings.color_depth(ctx.caps) != ColorDepth::Ansi16
}

fn arrows(unicode: bool) -> [&'static str; 3] {
    if unicode {
        ["◄", "►", "▼"]
    } else {
        ["<", ">", "v"]
    }
}

fn columns_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn pixel(rows: &[[Rgb; PIXELS]; PIXELS], row: usize, column: usize) -> Rgb {
    rows.get(row)
        .and_then(|cells| cells.get(column))
        .copied()
        .unwrap_or([0, 0, 0])
}

fn flag_spans(locale: Locale, half: usize, ctx: &ViewCtx<'_>) -> Vec<Span<'static>> {
    let depth = ctx.settings.color_depth(ctx.caps);
    let rows = flag(locale).pixels;
    let color = |[r, g, b]: Rgb| swatch(r, g, b, Color::White).resolve(depth);
    (0..PIXELS)
        .map(|column| {
            let style = Style::new()
                .fg(color(pixel(&rows, half * 2, column)))
                .bg(color(pixel(&rows, half * 2 + 1, column)));
            Span::styled("▀", style)
        })
        .collect()
}

fn border_set(ctx: &ViewCtx<'_>) -> border::Set<'static> {
    if ctx.glyphs.unicode {
        border::PLAIN
    } else {
        ASCII_BORDER
    }
}

fn boxed(ctx: &ViewCtx<'_>) -> Block<'static> {
    Block::bordered()
        .border_set(border_set(ctx))
        .border_style(ctx.theme.style(Token::BorderFocus, Token::Surface))
        .style(ctx.theme.surface())
}

fn locate(origin: Rect, flags: bool) -> Spots {
    let cell = |column: u16, height: u16| Rect::new(origin.x + column, origin.y, 1, height);
    let columns = if flags {
        [(0, origin.height), (7, 1), (9, origin.height)]
    } else {
        [(0, 1), (5, 1), (7, 1)]
    };
    let [left, right, down] = columns.map(|(column, rows)| cell(column, rows));
    Spots {
        body: origin,
        left,
        right,
        down,
        ..Spots::default()
    }
}

fn selector_lines(
    current: Locale,
    focused: bool,
    flags: bool,
    ctx: &ViewCtx<'_>,
) -> Vec<Line<'static>> {
    let [left, right, down] = arrows(ctx.glyphs.unicode);
    let style = if focused {
        ctx.theme.focus()
    } else {
        ctx.theme.style(Token::Accent, Token::Background)
    };
    let code = flag(current).code;
    if !flags {
        return vec![Line::styled(format!("{left} {code} {right} {down}"), style)];
    }
    let mut top = vec![Span::styled(format!("{left} "), style)];
    top.extend(flag_spans(current, 0, ctx));
    top.push(Span::styled(format!(" {right} {down}"), style));
    let mut bottom = vec![Span::styled("  ", style)];
    bottom.extend(flag_spans(current, 1, ctx));
    bottom.push(Span::styled(format!(" {code} "), style));
    vec![Line::from(top), Line::from(bottom)]
}

struct Entry {
    locale: Locale,
    highlighted: bool,
    current: bool,
    width: u16,
    flags: bool,
}

fn padded(text: String, width: u16, used: u16) -> String {
    let filler = " ".repeat(usize::from(width.saturating_sub(used)));
    format!("{text}{filler}")
}

fn entry_lines(entry: &Entry, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
    let style = if entry.highlighted {
        ctx.theme.focus()
    } else {
        ctx.theme.surface()
    };
    let mark = marker(ctx, entry.highlighted);
    let name = entry.locale.own_name();
    let code = flag(entry.locale).code;
    let sign = if entry.current {
        ctx.glyphs.selected
    } else {
        " "
    };
    if !entry.flags {
        let text = format!("{mark} {code} {name} {sign}");
        let used = columns_of(&text);
        return vec![Line::styled(padded(text, entry.width, used), style)];
    }
    let lead = 2 + FLAG_CELLS;
    let first = format!(" {name} {sign}");
    let second = format!(" {code}");
    let mut top = vec![Span::styled(format!("{mark} "), style)];
    top.extend(flag_spans(entry.locale, 0, ctx));
    let used = lead + columns_of(&first);
    top.push(Span::styled(padded(first, entry.width, used), style));
    let mut bottom = vec![Span::styled("  ", style)];
    bottom.extend(flag_spans(entry.locale, 1, ctx));
    let used = lead + columns_of(&second);
    bottom.push(Span::styled(padded(second, entry.width, used), style));
    vec![Line::from(top), Line::from(bottom)]
}

impl LanguagePicker {
    /// Draws the selector in the top-left corner of `area`, then its list while open, else its
    /// tooltip while the pointer is over it or it is `focused`.
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        current: Locale,
        focused: bool,
        ctx: &ViewCtx<'_>,
    ) {
        let flags = flags_shown(ctx);
        let (width, height) = if flags {
            BODY_WITH_FLAG
        } else {
            BODY_WITH_CODE
        };
        let origin = Rect::new(area.x + 1, area.y + 1, width, height).intersection(area);
        self.spots = locate(origin, flags);
        let lines = selector_lines(current, focused, flags, ctx);
        frame.render_widget(Paragraph::new(lines), origin);
        if let Some(highlighted) = self.open {
            self.draw_list(frame, area, highlighted, current, flags, ctx);
        } else if focused || self.pointer.is_some_and(|cell| contains(origin, cell)) {
            draw_tooltip(frame, area, origin, current, ctx);
        }
    }

    fn draw_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        highlighted: usize,
        current: Locale,
        flags: bool,
        ctx: &ViewCtx<'_>,
    ) {
        let rows: u16 = if flags { 2 } else { 1 };
        let languages = Locale::offered();
        let name_width = languages
            .iter()
            .map(|locale| columns_of(locale.own_name()))
            .max()
            .unwrap_or(0);
        let inner_width = name_width + if flags { 9 } else { 7 };
        let count = u16::try_from(languages.len()).unwrap_or(0);
        let outer = Rect::new(
            self.spots.body.x,
            self.spots.body.bottom(),
            inner_width + 2,
            count * rows + 2,
        )
        .intersection(area);
        frame.render_widget(Clear, outer);
        let block = boxed(ctx);
        let inner = block.inner(outer);
        frame.render_widget(block, outer);
        self.spots.list = outer;
        self.spots.entries.clear();
        for (index, locale) in languages.iter().enumerate() {
            let offset = u16::try_from(index).unwrap_or(0) * rows;
            let rect = Rect::new(inner.x, inner.y + offset, inner.width, rows).intersection(inner);
            let entry = Entry {
                locale: *locale,
                highlighted: index == highlighted,
                current: index == index_of(current),
                width: rect.width,
                flags,
            };
            frame.render_widget(Paragraph::new(entry_lines(&entry, ctx)), rect);
            self.spots.entries.push(rect);
        }
    }
}

fn draw_tooltip(frame: &mut Frame, area: Rect, body: Rect, current: Locale, ctx: &ViewCtx<'_>) {
    let name = current.own_name();
    let width = columns_of(name) + 4;
    let rect = Rect::new(body.x, body.bottom(), width, 3).intersection(area);
    frame.render_widget(Clear, rect);
    let text =
        Paragraph::new(Line::styled(format!(" {name} "), ctx.theme.surface())).block(boxed(ctx));
    frame.render_widget(text, rect);
}

#[cfg(test)]
mod tests;
