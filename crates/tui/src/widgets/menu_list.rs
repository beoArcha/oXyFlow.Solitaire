//! A vertical list with one focused item: the base of the menus. The focused item has a marker
//! directly before its label and reverse video, so the focus is visible without color. Up and Down
//! move it, and so do Tab and Shift+Tab, wrapping at both ends. Keys, the wheel and clicks all work.
//! A row may carry a muted detail after its label and an empty row below it; neither takes focus.

use ratatui::Frame;
use ratatui::crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::mouse::{contains, left_press, wheel_action};
use crate::screens::ViewCtx;
use crate::screens::focus::{marker, step, tab_forward};
use crate::theme::Token;

/// How the focused item of a list is shown, which depends on where the focus of the screen is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    /// The list holds the focus: the focused item is marked and drawn in the focus style.
    Focused,
    /// The focus is in another pane: the focused item keeps its marker, drawn in muted text.
    Muted,
    /// The list does not hold the focus: no item is marked.
    Off,
}

/// One item of a list: its label, an optional detail drawn after the label, and whether an empty
/// row follows it. The detail is muted unless the item is selected.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Row {
    label: String,
    detail: Option<String>,
    gap_after: bool,
}

impl Row {
    /// A row with `label` only.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            gap_after: false,
        }
    }

    /// The same row with `detail` drawn after the label.
    pub fn with_detail(self, detail: impl Into<String>) -> Self {
        Self {
            detail: Some(detail.into()),
            ..self
        }
    }

    /// The same row followed by one empty row, which takes neither focus nor clicks.
    pub fn with_gap_after(self) -> Self {
        Self {
            gap_after: true,
            ..self
        }
    }

    fn width(&self) -> usize {
        let detail = self.detail.as_ref().map_or(0, |text| text.chars().count());
        self.label.chars().count() + detail
    }

    fn height(&self) -> u16 {
        1 + u16::from(self.gap_after)
    }
}

/// Focus state and the screen rectangles of the items drawn last.
#[derive(Debug, Default)]
pub struct MenuList {
    focus: usize,
    hits: Vec<Rect>,
}

fn forward_of(action: Action) -> Option<bool> {
    match action {
        Action::Down => Some(true),
        Action::Up => Some(false),
        other => tab_forward(other),
    }
}

impl MenuList {
    /// A list focused on its first item.
    pub fn new() -> Self {
        Self::default()
    }

    /// The rows of `rows` drawn one under the other, empty rows included.
    pub fn height(rows: &[Row]) -> u16 {
        rows.iter()
            .fold(0, |sum, row| sum.saturating_add(row.height()))
    }

    /// The index of the focused item.
    pub fn focus(&self) -> usize {
        self.focus
    }

    /// Moves focus up or down, or with Tab and Shift+Tab (wrapping). Returns the index when the
    /// focused item is activated.
    pub fn on_action(&mut self, action: Action, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        if let Some(forward) = forward_of(action) {
            self.focus = step(self.focus.min(len - 1), len, forward);
        }
        (action == Action::Select).then_some(self.focus)
    }

    /// Handles the wheel and left clicks. Returns the index of a clicked item.
    pub fn on_mouse(&mut self, event: &MouseEvent, len: usize) -> Option<usize> {
        if let Some(action) = wheel_action(event) {
            self.on_action(action, len);
            return None;
        }
        let cell = left_press(event)?;
        let index = self.hits.iter().position(|hit| contains(*hit, cell))?;
        self.focus = index;
        Some(index)
    }

    /// Draws the rows centered in `area`, one row each (plus the empty rows), and remembers where
    /// the items are. The focused item is marked unless `highlight` is `Off`, and drawn in the focus
    /// style only when `highlight` is `Focused`.
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        rows: &[Row],
        ctx: &ViewCtx<'_>,
        highlight: Highlight,
    ) {
        let widest = rows.iter().map(Row::width).max().unwrap_or(0);
        let width = u16::try_from(widest + 2)
            .unwrap_or(u16::MAX)
            .min(area.width);
        let x = area.x + (area.width - width) / 2;
        let focus = self.focus.min(rows.len().saturating_sub(1));
        self.hits.clear();
        let mut offset: u16 = 0;
        for (index, row) in rows.iter().enumerate() {
            if offset >= area.height {
                break;
            }
            let rect = Rect::new(x, area.y + offset, width, 1);
            let is_focus = index == focus;
            let focused = highlight != Highlight::Off && is_focus;
            let line = item_line(ctx, highlight, is_focus, focused, row);
            frame.render_widget(Paragraph::new(line), rect);
            self.hits.push(rect);
            offset = offset.saturating_add(row.height());
        }
    }
}

fn item_line(
    ctx: &ViewCtx<'_>,
    highlight: Highlight,
    is_focus: bool,
    focused: bool,
    row: &Row,
) -> Line<'static> {
    let style = item_style(ctx, highlight, is_focus);
    let head = format!("{} {}", marker(ctx, focused), row.label);
    match &row.detail {
        Some(detail) => Line::from(vec![
            Span::styled(head, style),
            Span::styled(detail.clone(), detail_style(ctx, style, is_focus)),
        ]),
        None => Line::styled(head, style),
    }
}

fn item_style(ctx: &ViewCtx<'_>, highlight: Highlight, is_focus: bool) -> Style {
    match (highlight, is_focus) {
        (Highlight::Focused, true) => ctx.theme.focus(),
        (Highlight::Muted, true) => ctx.theme.style(Token::TextMuted, Token::Background),
        _ => ctx.theme.base(),
    }
}

fn detail_style(ctx: &ViewCtx<'_>, selected: Style, is_focus: bool) -> Style {
    if is_focus {
        selected
    } else {
        ctx.theme.style(Token::TextMuted, Token::Background)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn focus_wraps_in_both_directions() {
        let mut list = MenuList::new();
        assert_eq!(list.on_action(Action::Up, 3), None);
        assert_eq!(list.focus(), 2);
        list.on_action(Action::Down, 3);
        assert_eq!(list.focus(), 0);
    }

    #[test]
    fn tab_and_shift_tab_move_the_focus_and_wrap() {
        let mut list = MenuList::new();
        list.on_action(Action::FocusNext, 3);
        assert_eq!(list.focus(), 1);
        list.on_action(Action::FocusNext, 3);
        list.on_action(Action::FocusNext, 3);
        assert_eq!(list.focus(), 0);
        list.on_action(Action::FocusPrevious, 3);
        assert_eq!(list.focus(), 2);
    }

    #[test]
    fn tab_does_not_activate_the_focused_item() {
        let mut list = MenuList::new();
        assert_eq!(list.on_action(Action::FocusNext, 3), None);
    }

    #[test]
    fn select_activates_the_focused_item() {
        let mut list = MenuList::new();
        list.on_action(Action::Down, 3);
        assert_eq!(list.on_action(Action::Select, 3), Some(1));
    }

    #[test]
    fn an_empty_list_ignores_everything() {
        let mut list = MenuList::new();
        assert_eq!(list.on_action(Action::Select, 0), None);
        assert_eq!(
            list.on_mouse(&mouse(MouseEventKind::ScrollDown, 0, 0), 0),
            None
        );
    }

    #[test]
    fn the_wheel_moves_focus_without_activating() {
        let mut list = MenuList::new();
        assert_eq!(
            list.on_mouse(&mouse(MouseEventKind::ScrollDown, 0, 0), 3),
            None
        );
        assert_eq!(list.focus(), 1);
    }

    #[test]
    fn a_click_outside_every_item_does_nothing() {
        let mut list = MenuList::new();
        let click = mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
        assert_eq!(list.on_mouse(&click, 3), None);
        assert_eq!(list.focus(), 0);
    }

    #[test]
    fn an_empty_row_after_an_item_adds_one_row_of_height() {
        let rows = [
            Row::new("first").with_gap_after(),
            Row::new("second").with_detail(": muted"),
            Row::new("third"),
        ];
        assert_eq!(MenuList::height(&rows), 4);
        assert_eq!(MenuList::height(&[]), 0);
    }

    #[test]
    fn a_row_is_as_wide_as_its_label_and_its_detail() {
        let row = Row::new("Resume").with_detail(": Klondike");
        assert_eq!(row.width(), "Resume: Klondike".chars().count());
        assert_eq!(Row::new("New game").width(), "New game".chars().count());
    }
}
