//! The guide panel beside the board: the goal of the game in a line or two, the keys in groups
//! (generated from the key table) and the mouse gestures. Keys usable at this moment are marked
//! with a symbol and drawn bold, so they stand out without color.

use oxysol_core::Variant;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::text::{clip, wrap};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::{binding_for, primary_key};
use crate::screens::ViewCtx;
use crate::theme::Token;

const PADDING: usize = 1;
const EDGE_WIDTH: usize = 1;
const KEY_GAP: usize = 2;
const MARKER_WIDTH: usize = 2;
const MOUSE_CLICK: &str = "board.guide.mouse.click";
const MOUSE_DOUBLE: &str = "board.guide.mouse.double_click";
const MOUSE_STOCK: &str = "board.guide.mouse.stock";
const MOUSE_RIGHT: &str = "board.guide.mouse.right_click";

/// What the panel needs to know: the game being played and the actions usable right now.
#[derive(Debug, Clone, Copy)]
pub struct Guide<'a> {
    pub variant: Variant,
    pub usable: &'a [Action],
}

struct Entry {
    actions: &'static [Action],
    label: &'static str,
}

const fn entry(actions: &'static [Action], label: &'static str) -> Entry {
    Entry { actions, label }
}

fn entries(variant: Variant) -> Vec<Entry> {
    let pyramid = matches!(variant, Variant::Pyramid(_));
    let mut rows = vec![
        entry(
            &[Action::Up, Action::Down, Action::Left, Action::Right],
            "board.guide.move",
        ),
        entry(
            &[Action::Select],
            if pyramid {
                "board.guide.select_pair"
            } else {
                "board.guide.select"
            },
        ),
        entry(&[Action::Back], "board.guide.back"),
        entry(&[Action::QuickMove], "board.guide.quick"),
    ];
    if !pyramid {
        rows.push(entry(
            &[Action::RunMore, Action::RunLess],
            "board.guide.run",
        ));
    }
    match variant {
        Variant::FreeCell => {}
        Variant::Spider(_) => rows.push(entry(&[Action::Stock], "board.guide.deal_row")),
        _ => rows.push(entry(&[Action::Stock], "board.guide.stock")),
    }
    rows.extend([
        entry(&[Action::Undo, Action::Redo], "board.guide.undo_redo"),
        entry(&[Action::Hint], "board.guide.hint"),
        entry(&[Action::Rules], "board.guide.rules"),
        entry(&[Action::Help], "board.guide.help"),
    ]);
    rows
}

fn mouse_gestures(variant: Variant) -> Vec<&'static str> {
    let mut gestures = vec![MOUSE_CLICK, MOUSE_DOUBLE];
    gestures.extend((variant != Variant::FreeCell).then_some(MOUSE_STOCK));
    gestures.push(MOUSE_RIGHT);
    gestures
}

const fn goal_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Klondike(_) => "board.guide.goal.klondike",
        Variant::Spider(_) => "board.guide.goal.spider",
        Variant::FreeCell => "board.guide.goal.freecell",
        Variant::Pyramid(_) => "board.guide.goal.pyramid",
    }
}

fn is_arrow(action: Action) -> bool {
    binding_for(action)
        .and_then(|binding| binding.keys.first())
        .is_some_and(|spec| {
            matches!(
                spec.code,
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
            )
        })
}

/// The keys of a group of actions for display: `Arrows` when they are the four arrow keys, else
/// the primary key of each action joined by `/`.
fn keys_text(actions: &[Action]) -> String {
    if actions.len() > 1 && actions.iter().all(|action| is_arrow(*action)) {
        return t("board.guide.arrows").into_owned();
    }
    actions
        .iter()
        .filter_map(|action| primary_key(*action))
        .collect::<Vec<_>>()
        .join("/")
}

/// The row of the panel that carries the title; a click on it hides the panel.
pub fn title_row(area: Rect) -> Rect {
    Rect::new(area.x, area.y, area.width, area.height.min(1))
}

fn text_width(area: Rect) -> usize {
    usize::from(area.width).saturating_sub(PADDING + EDGE_WIDTH)
}

fn heading(key: &str, ctx: &ViewCtx<'_>) -> Line<'static> {
    let style = ctx
        .theme
        .style(Token::Accent, Token::Surface)
        .add_modifier(Modifier::BOLD);
    Line::styled(t(key).into_owned(), style)
}

fn row(
    entry: &Entry,
    key_width: usize,
    width: usize,
    guide: &Guide<'_>,
    ctx: &ViewCtx<'_>,
) -> Line<'static> {
    let usable = entry
        .actions
        .iter()
        .any(|action| guide.usable.contains(action));
    let (marker, style) = if usable {
        (
            ctx.glyphs.focus_left,
            ctx.theme
                .style(Token::Text, Token::Surface)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (" ", ctx.theme.style(Token::TextMuted, Token::Surface))
    };
    let keys = keys_text(entry.actions);
    let label_width = width.saturating_sub(MARKER_WIDTH + key_width + KEY_GAP);
    let text = format!(
        "{marker} {keys:<key_width$}{}{}",
        " ".repeat(KEY_GAP),
        clip(&t(entry.label), label_width)
    );
    Line::styled(text, style)
}

/// Every line of the panel for a text width of `width` columns.
pub fn lines(guide: &Guide<'_>, width: usize, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
    let plain = ctx.theme.style(Token::Text, Token::Surface);
    let muted: Style = ctx.theme.style(Token::TextMuted, Token::Surface);
    let hide = primary_key(Action::Guide).unwrap_or_default();
    let mut lines = vec![Line::styled(
        clip(&tf("board.guide.title", &[("key", &hide)]), width),
        plain.add_modifier(Modifier::BOLD),
    )];
    lines.push(heading("board.guide.goal_title", ctx));
    lines.extend(
        wrap(&t(goal_key(guide.variant)), width)
            .into_iter()
            .map(|text| Line::styled(text, plain)),
    );
    lines.push(Line::raw(String::new()));
    lines.push(heading("board.guide.keys_title", ctx));
    let rows = entries(guide.variant);
    let key_width = rows
        .iter()
        .map(|entry| keys_text(entry.actions).chars().count())
        .max()
        .unwrap_or(0);
    lines.extend(
        rows.iter()
            .map(|entry| row(entry, key_width, width, guide, ctx)),
    );
    lines.push(Line::raw(String::new()));
    lines.push(heading("board.guide.mouse_title", ctx));
    lines.extend(
        mouse_gestures(guide.variant)
            .iter()
            .map(|key| Line::styled(clip(&t(key), width), muted)),
    );
    lines
}

/// Draws the panel into `area`: an edge on the left, then the text, cut at the bottom.
pub fn draw(buf: &mut Buffer, area: Rect, guide: &Guide<'_>, ctx: &ViewCtx<'_>) {
    if area.width <= 3 || area.height == 0 {
        return;
    }
    Block::new()
        .style(ctx.theme.style(Token::Text, Token::Surface))
        .render(area, buf);
    let edge = ctx.theme.style(Token::Border, Token::Surface);
    for y in area.top()..area.bottom() {
        if let Some(cell) = buf.cell_mut((area.x, y)) {
            cell.set_symbol(ctx.glyphs.edge).set_style(edge);
        }
    }
    let inner = Rect::new(
        area.x + 1 + u16::try_from(PADDING).unwrap_or(1),
        area.y,
        area.width
            .saturating_sub(1 + u16::try_from(PADDING).unwrap_or(1)),
        area.height,
    );
    Paragraph::new(lines(guide, text_width(area), ctx)).render(inner, buf);
}
