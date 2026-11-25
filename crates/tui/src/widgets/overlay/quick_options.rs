//! The quick options dialog of the board: the aids (aura, target marks, guide panel, possible moves
//! panel, auto-finish) for the running game. A change takes effect at once: the dialog holds the
//! choices and the application copies them to the game after every key or click. "Save as default"
//! makes them the defaults of the Settings screen. Rows, "Save as default" and Close form one focus
//! ring (Tab); Up and Down walk it too, Left and Right change the focused row or move between the
//! two buttons.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::dialog::{self, Button, Geometry};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::mouse::{self, contains};
use crate::screens::focus::{marker, step, tab_forward};
use crate::screens::{Effect, ViewCtx};
use crate::settings::{Aid, Aids};
use crate::theme::Token;

const BOX_WIDTH: u16 = 66;
const AID_ROWS: u16 = Aid::ALL.len() as u16;
const DESCRIPTION_ROWS: u16 = 2;
const BODY_ROWS: u16 = 1 + 1 + AID_ROWS + 1 + DESCRIPTION_ROWS + 1;
const SAVE: usize = Aid::ALL.len();
const CLOSE: usize = SAVE + 1;
const RING: usize = CLOSE + 1;

/// The open dialog: the choices of the running game and the element that has the focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickOptions {
    pub aids: Aids,
    pub focus: usize,
}

impl QuickOptions {
    /// The dialog over the choices of the running game, focused on the first row.
    pub fn new(aids: Aids) -> Self {
        Self { aids, focus: 0 }
    }

    fn aid(&self) -> Option<Aid> {
        Aid::ALL.get(self.focus).copied()
    }

    fn switch(&mut self, index: usize) {
        if let Some(aid) = Aid::ALL.get(index) {
            self.aids = aid.toggled(&self.aids);
        }
    }

    fn press(&mut self) -> Option<Effect> {
        match self.focus {
            SAVE => Some(Effect::SaveAids),
            CLOSE => Some(Effect::None),
            index => {
                self.switch(index);
                None
            }
        }
    }

    fn sideways(&mut self) {
        match self.focus {
            SAVE => self.focus = CLOSE,
            CLOSE => self.focus = SAVE,
            index => self.switch(index),
        }
    }
}

fn buttons(state: &QuickOptions) -> Vec<Button> {
    let close = tf(
        "help.rules.close",
        &[("key", &primary_key(Action::Back).unwrap_or_default())],
    );
    vec![
        Button {
            label: t("board.options.save").into_owned(),
            focused: state.focus == SAVE,
        },
        Button {
            label: close,
            focused: state.focus == CLOSE,
        },
    ]
}

/// The box of the dialog, centered in `area`.
pub fn geometry(area: Rect, state: &QuickOptions) -> Geometry {
    dialog::boxed(area, BOX_WIDTH, BODY_ROWS, &buttons(state))
}

struct Parts {
    note: Rect,
    rows: Vec<Rect>,
    description: Rect,
    footer: Rect,
}

fn parts(body: Rect) -> Parts {
    let [note, _, list, _, description, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(AID_ROWS),
        Constraint::Length(1),
        Constraint::Length(DESCRIPTION_ROWS),
        Constraint::Length(1),
    ])
    .areas(body.inner(Margin::new(1, 0)));
    let rows = (0..AID_ROWS)
        .map(|index| Rect::new(list.x, list.y.saturating_add(index), list.width, 1))
        .collect();
    Parts {
        note,
        rows,
        description,
        footer,
    }
}

fn label_width() -> usize {
    Aid::ALL
        .iter()
        .map(|aid| t(aid.label_key()).chars().count())
        .max()
        .unwrap_or_default()
}

fn row_text(
    aid: Aid,
    state: &QuickOptions,
    focused: bool,
    columns: u16,
    ctx: &ViewCtx<'_>,
) -> String {
    let name = t(aid.label_key()).into_owned();
    let leaders = ".".repeat(label_width().saturating_sub(name.chars().count()) + 2);
    let value = aid.value_for(&state.aids, columns);
    format!("{} {name} {leaders} {value}", marker(ctx, focused))
}

fn row_style(focused: bool, ctx: &ViewCtx<'_>) -> Style {
    let style = ctx.theme.style(Token::Text, Token::Surface);
    if focused {
        style.add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        style
    }
}

fn footer_text() -> String {
    let key = |action| primary_key(action).unwrap_or_default();
    tf(
        "board.options.footer",
        &[
            ("up", &key(Action::Up)),
            ("down", &key(Action::Down)),
            ("left", &key(Action::Left)),
            ("right", &key(Action::Right)),
            ("select", &key(Action::Select)),
        ],
    )
}

/// Draws the dialog over `area`.
pub fn draw(frame: &mut Frame, area: Rect, state: &QuickOptions, ctx: &ViewCtx<'_>) {
    let geometry = geometry(area, state);
    dialog::draw_box(frame, &geometry, &t("board.options.title"), ctx);
    let parts = parts(geometry.body);
    let muted = ctx.theme.style(Token::TextMuted, Token::Surface);
    frame.render_widget(
        Paragraph::new(Line::styled(t("board.options.note").into_owned(), muted)),
        parts.note,
    );
    for (index, (aid, row)) in Aid::ALL.iter().zip(&parts.rows).enumerate() {
        let focused = index == state.focus;
        let text = row_text(*aid, state, focused, area.width, ctx);
        frame.render_widget(
            Paragraph::new(Line::styled(text, row_style(focused, ctx))),
            *row,
        );
    }
    if let Some(aid) = state.aid() {
        frame.render_widget(
            Paragraph::new(t(aid.description_key()).into_owned())
                .wrap(Wrap { trim: true })
                .style(muted),
            parts.description,
        );
    }
    frame.render_widget(
        Paragraph::new(Line::styled(footer_text(), muted)).alignment(Alignment::Center),
        parts.footer,
    );
    dialog::draw_buttons(frame, &geometry, &buttons(state), ctx);
}

/// Handles a key while the dialog is open. `None` keeps it open; `Some(effect)` closes it.
pub fn handle(state: &mut QuickOptions, _key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    let action = action?;
    if let Some(forward) = tab_forward(action) {
        state.focus = step(state.focus, RING, forward);
        return None;
    }
    match action {
        Action::Up => state.focus = step(state.focus, RING, false),
        Action::Down => state.focus = step(state.focus, RING, true),
        Action::Left | Action::Right => state.sideways(),
        Action::Select => return state.press(),
        Action::Back | Action::Options => return Some(Effect::None),
        _ => {}
    }
    None
}

/// Handles a mouse event: a click on a row focuses and switches it, a click on a button presses it.
pub fn handle_mouse(state: &mut QuickOptions, area: Rect, event: &MouseEvent) -> Option<Effect> {
    let cell = mouse::left_press(event)?;
    let geometry = geometry(area, state);
    if let Some(button) = dialog::button_at(&geometry, cell) {
        state.focus = SAVE + button;
        return state.press();
    }
    let index = parts(geometry.body)
        .rows
        .iter()
        .position(|row| contains(*row, cell))?;
    state.focus = index;
    state.switch(index);
    None
}
