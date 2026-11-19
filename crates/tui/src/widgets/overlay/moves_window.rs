//! The possible moves window: the entries of the side panel as a box over the board, for screens
//! narrower than the panel needs. Up and Down (or Tab) choose an entry, Enter or a click runs an
//! active one and closes the window, an inactive one does nothing; the moves key, Esc or the Close
//! button close it. The entries are fixed when the window opens: the board stands still behind it.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::dialog::{self, Geometry};
use super::help;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::mouse::{self, contains};
use crate::screens::focus::{step, tab_forward};
use crate::screens::{Effect, ViewCtx};
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap, Segment};
use crate::widgets::moves::{MoveEntry, row_text};

const BOX_WIDTH: u16 = 54;

/// The open window: the entries as they were when it opened, and the one that has the focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovesWindow {
    pub entries: Vec<MoveEntry>,
    pub focus: usize,
}

impl MovesWindow {
    /// The window over `entries`, focused on the first active one (else the first).
    pub fn new(entries: Vec<MoveEntry>) -> Self {
        let focus = entries.iter().position(|entry| entry.active).unwrap_or(0);
        Self { entries, focus }
    }

    fn rows(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).unwrap_or(u16::MAX)
    }

    fn focus_by(&mut self, step_forward: bool) {
        self.focus = step(self.focus, self.entries.len(), step_forward);
    }

    fn run_focused(&self) -> Option<Effect> {
        let entry = self.entries.get(self.focus).filter(|entry| entry.active)?;
        Some(Effect::Board(entry.action))
    }
}

/// The box of the window, centered in `area`; it is as tall as the list.
pub fn geometry(area: Rect, state: &MovesWindow) -> Geometry {
    dialog::boxed(area, BOX_WIDTH, state.rows(), &help::buttons())
}

fn split(body: Rect) -> (Rect, Rect) {
    let [text, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(body);
    (text, footer)
}

fn entry_rows(area: Rect, state: &MovesWindow) -> Vec<Rect> {
    let text = split(geometry(area, state).body).0.inner(Margin::new(1, 0));
    (0..state.entries.len())
        .map(|index| {
            let y = text
                .y
                .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
            Rect::new(text.x, y, text.width, 1).intersection(text)
        })
        .collect()
}

fn with_reason(entry: &MoveEntry) -> MoveEntry {
    let label = match &entry.reason {
        Some(reason) => format!("{}: {reason}", entry.label),
        None => entry.label.clone(),
    };
    MoveEntry {
        label,
        ..entry.clone()
    }
}

fn key_name(action: Action) -> String {
    primary_key(action).unwrap_or_default()
}

fn entry_style(entry: &MoveEntry, focused: bool, ctx: &ViewCtx<'_>) -> Style {
    let style = if entry.active {
        ctx.theme
            .style(Token::Text, Token::Surface)
            .add_modifier(Modifier::BOLD)
    } else {
        ctx.theme
            .style(Token::TextMuted, Token::Surface)
            .add_modifier(Modifier::DIM)
    };
    if focused {
        style.add_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

/// Draws the window over `area` and returns its footer buttons.
pub fn draw(frame: &mut Frame, area: Rect, state: &MovesWindow, ctx: &ViewCtx<'_>) -> FooterMap {
    let geometry = geometry(area, state);
    dialog::draw_box(frame, &geometry, &t("board.moves.window_title"), ctx);
    let (_, footer_row) = split(geometry.body);
    for ((index, entry), row) in state
        .entries
        .iter()
        .enumerate()
        .zip(entry_rows(area, state))
    {
        let focused = index == state.focus;
        let text = row_text(&with_reason(entry), focused, usize::from(row.width), ctx);
        frame.render_widget(
            Paragraph::new(Line::styled(text, entry_style(entry, focused, ctx))),
            row,
        );
    }
    let hint = tf(
        "board.moves.window_footer",
        &[
            ("up", &key_name(Action::Up)),
            ("down", &key_name(Action::Down)),
            ("select", &key_name(Action::Select)),
        ],
    );
    let close = tf(
        "board.moves.window_close",
        &[
            ("key", &key_name(Action::Moves)),
            ("back", &key_name(Action::Back)),
        ],
    );
    let segments = [
        Segment::plain(hint),
        Segment::plain("   "),
        Segment::button(Action::Moves, close),
    ];
    let muted = ctx.theme.style(Token::TextMuted, Token::Surface);
    let buttons = footer::draw(
        frame,
        footer_row.inner(Margin::new(1, 0)),
        &segments,
        Alignment::Left,
        muted,
        ctx,
    );
    dialog::draw_buttons(frame, &geometry, &help::buttons(), ctx);
    buttons
}

/// Handles a key while the window is open. `None` keeps it open; `Some(effect)` closes it.
pub fn handle(state: &mut MovesWindow, _key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    let action = action?;
    if let Some(forward) = tab_forward(action) {
        state.focus_by(forward);
        return None;
    }
    match action {
        Action::Up => state.focus = state.focus.saturating_sub(1),
        Action::Down => {
            state.focus = (state.focus + 1).min(state.entries.len().saturating_sub(1));
        }
        Action::Select => return state.run_focused(),
        Action::Back | Action::Moves => return Some(Effect::None),
        _ => {}
    }
    None
}

/// Handles a mouse event: a click on an active entry runs it, a click on an inactive one only
/// focuses it, a click on Close closes the window.
pub fn handle_mouse(state: &mut MovesWindow, area: Rect, event: &MouseEvent) -> Option<Effect> {
    let cell = mouse::left_press(event)?;
    let hit = entry_rows(area, state)
        .iter()
        .position(|row| contains(*row, cell));
    if let Some(index) = hit {
        state.focus = index;
        return state.run_focused();
    }
    dialog::button_at(&geometry(area, state), cell).map(|_| Effect::None)
}
