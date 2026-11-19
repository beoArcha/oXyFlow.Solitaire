//! The guide panel beside the board: a framed panel with its title in the top border, a `Game`
//! compartment with the game information on top, then the `Goal`, `Keys`, `Mouse` and `Legend`
//! sections. It never repeats a fact of the controls strip under the board: what is focused and
//! what can be done now live there, the panel holds the game facts and the fixed reference.

mod keys;
mod legend;

use oxysol_core::Variant;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::text::clip;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// Columns the frame and the padding take on each side of the text.
const SIDE: u16 = 2;
/// Rows the frame takes at the top and at the bottom.
const FRAME_ROWS: u16 = 2;

const ASCII_BORDER: border::Set<'static> = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

/// What the panel needs to know: the game being played, the actions usable right now and the
/// lines of the `Game` compartment.
#[derive(Debug, Clone, Copy)]
pub struct Guide<'a> {
    pub variant: Variant,
    pub usable: &'a [Action],
    /// Whether the game allows undo and redo; they are not listed when it does not.
    pub undo_allowed: bool,
    /// The game information, one line per fact, below the `Game` heading.
    pub game: &'a [Line<'static>],
}

/// The row of the panel that carries the title, its top border; a click on it hides the panel.
pub fn title_row(area: Rect) -> Rect {
    Rect::new(area.x, area.y, area.width, area.height.min(1))
}

/// Where the first fact of the `Game` compartment is drawn: the text area of the compartment,
/// starting under its heading. The board uses it to find the seed under a click.
pub fn game_text(area: Rect) -> Rect {
    Rect::new(
        area.x + SIDE,
        area.y + 2,
        area.width.saturating_sub(2 * SIDE),
        area.height.saturating_sub(FRAME_ROWS + 1),
    )
}

fn border_set(ctx: &ViewCtx<'_>) -> border::Set<'static> {
    if ctx.glyphs.unicode {
        border::PLAIN
    } else {
        ASCII_BORDER
    }
}

fn heading(key: &str, ctx: &ViewCtx<'_>) -> Line<'static> {
    let style = ctx
        .theme
        .style(Token::Accent, Token::Surface)
        .add_modifier(Modifier::BOLD);
    Line::styled(t(key).into_owned(), style)
}

fn section(key: &str, body: Vec<Line<'static>>, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
    let mut lines = vec![heading(key, ctx)];
    lines.extend(body);
    lines
}

/// The sections one after another, with a blank row between them when all of them still fit.
fn stacked(sections: Vec<Vec<Line<'static>>>, rows: usize) -> Vec<Line<'static>> {
    let total: usize = sections.iter().map(Vec::len).sum();
    let spaced = total + sections.len().saturating_sub(1) <= rows;
    let mut lines = Vec::new();
    for (index, body) in sections.into_iter().enumerate() {
        if spaced && index > 0 {
            lines.push(Line::raw(String::new()));
        }
        lines.extend(body);
    }
    lines
}

fn draw_frame(buf: &mut Buffer, area: Rect, ctx: &ViewCtx<'_>) {
    let hide = primary_key(Action::Guide).unwrap_or_default();
    let title = clip(
        &tf("board.guide.title", &[("key", &hide)]),
        usize::from(area.width.saturating_sub(SIDE * 2)),
    );
    Block::bordered()
        .border_set(border_set(ctx))
        .border_style(ctx.theme.style(Token::Border, Token::Surface))
        .title(Line::styled(
            format!(" {title} "),
            ctx.theme
                .style(Token::Accent, Token::Surface)
                .add_modifier(Modifier::BOLD),
        ))
        .style(ctx.theme.style(Token::Text, Token::Surface))
        .render(area, buf);
}

fn draw_rule(buf: &mut Buffer, area: Rect, row: u16, ctx: &ViewCtx<'_>) {
    let style = ctx.theme.style(Token::Border, Token::Surface);
    let (left, right, fill) = if ctx.glyphs.unicode {
        ("├", "┤", "─")
    } else {
        ("+", "+", "-")
    };
    for x in area.left()..area.right() {
        let symbol = if x == area.left() {
            left
        } else if x + 1 == area.right() {
            right
        } else {
            fill
        };
        if let Some(cell) = buf.cell_mut((x, row)) {
            cell.set_symbol(symbol).set_style(style);
        }
    }
}

/// Draws the panel into `area`: the frame, the `Game` compartment, a rule, then the sections,
/// cut at the bottom when the panel is shorter than they are.
pub fn draw(buf: &mut Buffer, area: Rect, guide: &Guide<'_>, ctx: &ViewCtx<'_>) {
    if area.width <= 2 * SIDE || area.height <= FRAME_ROWS {
        return;
    }
    draw_frame(buf, area, ctx);
    let width = usize::from(area.width - 2 * SIDE);
    let mut game = vec![heading("board.guide.game_title", ctx)];
    game.extend(guide.game.iter().cloned());
    let rule = area.y + 1 + u16::try_from(game.len()).unwrap_or(u16::MAX);
    let last = area.bottom().saturating_sub(1);
    let top = Rect::new(
        area.x + SIDE,
        area.y + 1,
        area.width - 2 * SIDE,
        rule.min(last) - area.y - 1,
    );
    Paragraph::new(game).render(top, buf);
    if rule + 1 >= last {
        return;
    }
    draw_rule(
        buf,
        Rect::new(area.x, area.y, area.width, area.height),
        rule,
        ctx,
    );
    let rest = Rect::new(top.x, rule + 1, top.width, last - rule - 1);
    let sections = vec![
        section(
            "board.guide.goal_title",
            keys::goal(guide.variant, width, ctx),
            ctx,
        ),
        section("board.guide.keys_title", keys::keys(guide, width, ctx), ctx),
        section(
            "board.guide.mouse_title",
            keys::mouse(guide.variant, width, ctx),
            ctx,
        ),
        section("board.guide.legend_title", legend::lines(width, ctx), ctx),
    ];
    Paragraph::new(stacked(sections, usize::from(rest.height))).render(rest, buf);
}
