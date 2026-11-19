//! The possible moves panel: a framed list of every board action as a button. An action that can
//! be run now is active (bold, `•`); one that cannot stays listed, dimmed and marked `×`, with its
//! reason on the row below when the panel is tall enough. The same entries fill the moves window of
//! a Compact screen. The list is built by the board from the legal moves; nothing is decided here.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;

use super::guide::{FRAME_ROWS, SIDE, draw_titled_frame};
use super::text::clip;
use crate::action::Action;
use crate::i18n::tf;
use crate::input::primary_key;
use crate::mouse::contains;
use crate::screens::ViewCtx;
use crate::screens::focus::marker;
use crate::theme::Token;

/// One board action in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveEntry {
    pub action: Action,
    pub label: String,
    pub key: String,
    pub active: bool,
    /// Why the entry is inactive; `None` for an active entry.
    pub reason: Option<String>,
}

/// The active entries of the last draw: the rows a click runs.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PanelMap {
    hits: Vec<(Rect, Action)>,
}

impl PanelMap {
    /// The action of the active entry at `cell`; inactive entries and the frame give `None`.
    pub fn action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.hits
            .iter()
            .find(|(rect, _)| contains(*rect, cell))
            .map(|(_, action)| *action)
    }
}

/// The row of the panel that carries the title, its top border; a click on it hides the panel.
pub fn title_row(area: Rect) -> Rect {
    Rect::new(area.x, area.y, area.width, area.height.min(1))
}

/// The text of the entry row: marker, sign, label, and the key at the right end. The label is
/// clipped so that the key always stays.
pub fn row_text(entry: &MoveEntry, focused: bool, width: usize, ctx: &ViewCtx<'_>) -> String {
    let sign = if entry.active {
        ctx.glyphs.entry_on
    } else {
        ctx.glyphs.entry_off
    };
    let head = format!("{}{sign} ", marker(ctx, focused));
    let key_width = entry.key.chars().count();
    let room = width.saturating_sub(head.chars().count() + key_width + 1);
    let label = clip(&entry.label, room);
    let used = head.chars().count() + label.chars().count() + key_width;
    let gap = " ".repeat(width.saturating_sub(used).max(1));
    format!("{head}{label}{gap}{}", entry.key)
}

fn entry_style(entry: &MoveEntry, focused: bool, hovered: bool, ctx: &ViewCtx<'_>) -> Style {
    let style = if entry.active {
        ctx.theme
            .style(Token::Text, Token::Surface)
            .add_modifier(Modifier::BOLD)
    } else {
        ctx.theme
            .style(Token::TextMuted, Token::Surface)
            .add_modifier(Modifier::DIM)
    };
    if focused || hovered {
        style.add_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

/// Whether every inactive entry can show its reason in `rows` rows.
pub fn reasons_fit(entries: &[MoveEntry], rows: usize) -> bool {
    let reasons = entries.iter().filter(|entry| !entry.active).count();
    entries.len() + reasons <= rows
}

fn draw_row(buf: &mut Buffer, row: Rect, text: &str, style: Style) {
    Span::styled(text.to_owned(), style).render(row, buf);
}

/// Draws the panel into `area`: the frame with its title, then one row per entry (and one per
/// reason when they fit). `focus` is the entry that has the focus. Returns the active rows.
pub fn draw(
    buf: &mut Buffer,
    area: Rect,
    entries: &[MoveEntry],
    focus: Option<usize>,
    ctx: &ViewCtx<'_>,
) -> PanelMap {
    let mut map = PanelMap::default();
    if area.width <= 2 * SIDE || area.height <= FRAME_ROWS {
        return map;
    }
    let hide = primary_key(Action::Moves).unwrap_or_default();
    draw_titled_frame(buf, area, &tf("board.moves.title", &[("key", &hide)]), ctx);
    let inner = Rect::new(
        area.x + SIDE,
        area.y + 1,
        area.width - 2 * SIDE,
        area.height - FRAME_ROWS,
    );
    let reasons = reasons_fit(entries, usize::from(inner.height));
    let width = usize::from(inner.width);
    let mut y = inner.y;
    for (index, entry) in entries.iter().enumerate() {
        if y >= inner.bottom() {
            break;
        }
        let row = Rect::new(inner.x, y, inner.width, 1);
        let focused = focus == Some(index);
        let hovered = entry.active && ctx.pointer.is_some_and(|cell| contains(row, cell));
        let text = row_text(entry, focused, width, ctx);
        draw_row(buf, row, &text, entry_style(entry, focused, hovered, ctx));
        if entry.active {
            map.hits.push((row, entry.action));
        }
        y += 1;
        let reason = entry
            .reason
            .as_deref()
            .filter(|_| reasons && y < inner.bottom());
        if let Some(reason) = reason {
            let indent = marker(ctx, false).chars().count() + 2;
            let text = format!("{}{}", " ".repeat(indent), clip(reason, width - indent));
            let muted = ctx
                .theme
                .style(Token::TextMuted, Token::Surface)
                .add_modifier(Modifier::DIM);
            draw_row(buf, Rect::new(inner.x, y, inner.width, 1), &text, muted);
            y += 1;
        }
    }
    map
}
