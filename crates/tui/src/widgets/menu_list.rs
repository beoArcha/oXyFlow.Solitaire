//! A vertical list with one focused item: the base of the menus. The focused item has a marker
//! directly before its label and reverse video, so the focus is visible without color. Up and Down
//! move it, and so do Tab and Shift+Tab, wrapping at both ends. Keys, the wheel and clicks all work.

use ratatui::Frame;
use ratatui::crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::mouse::{contains, left_press, wheel_action};
use crate::screens::ViewCtx;
use crate::screens::focus::{marker, step, tab_forward};

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

    /// Draws the items centered in `area`, one row each, and remembers where they are. The
    /// focused item is marked and highlighted only while `active`, that is while the list holds
    /// the focus of its screen.
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        labels: &[String],
        ctx: &ViewCtx<'_>,
        active: bool,
    ) {
        let widest = labels
            .iter()
            .map(|label| label.chars().count())
            .max()
            .unwrap_or(0);
        let width = u16::try_from(widest + 2)
            .unwrap_or(u16::MAX)
            .min(area.width);
        let x = area.x + (area.width - width) / 2;
        let focus = self.focus.min(labels.len().saturating_sub(1));
        self.hits.clear();
        for (index, label) in labels.iter().enumerate() {
            let Ok(offset) = u16::try_from(index) else {
                break;
            };
            if offset >= area.height {
                break;
            }
            let row = Rect::new(x, area.y + offset, width, 1);
            let focused = active && index == focus;
            let style = if focused {
                ctx.theme.focus()
            } else {
                ctx.theme.base()
            };
            let text = format!("{} {label}", marker(ctx, focused));
            frame.render_widget(Paragraph::new(Line::styled(text, style)), row);
            self.hits.push(row);
        }
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
}
