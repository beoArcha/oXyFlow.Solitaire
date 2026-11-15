//! Translation of mouse events into the actions the keyboard produces. Stub: P3-06 adds the board
//! gestures through `BoardGeometry::hit`; lists and menus already use these two helpers.

use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::action::Action;

/// The cell of a left-button press, if the event is one.
pub fn left_press(event: &MouseEvent) -> Option<(u16, u16)> {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => Some((event.column, event.row)),
        _ => None,
    }
}

/// The navigation action of a wheel turn: scrolling up is [`Action::Up`], down is [`Action::Down`].
pub fn wheel_action(event: &MouseEvent) -> Option<Action> {
    match event.kind {
        MouseEventKind::ScrollUp => Some(Action::Up),
        MouseEventKind::ScrollDown => Some(Action::Down),
        _ => None,
    }
}

/// Whether the cell lies inside the rectangle.
pub fn contains(area: Rect, cell: (u16, u16)) -> bool {
    let (column, row) = cell;
    column >= area.x && column < area.right() && row >= area.y && row < area.bottom()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn only_a_left_press_is_a_click() {
        let down = event(MouseEventKind::Down(MouseButton::Left), 3, 4);
        assert_eq!(left_press(&down), Some((3, 4)));
        assert_eq!(
            left_press(&event(MouseEventKind::Down(MouseButton::Right), 3, 4)),
            None
        );
        assert_eq!(
            left_press(&event(MouseEventKind::Up(MouseButton::Left), 3, 4)),
            None
        );
    }

    #[test]
    fn the_wheel_moves_up_and_down() {
        assert_eq!(
            wheel_action(&event(MouseEventKind::ScrollUp, 0, 0)),
            Some(Action::Up)
        );
        assert_eq!(
            wheel_action(&event(MouseEventKind::ScrollDown, 0, 0)),
            Some(Action::Down)
        );
        assert_eq!(wheel_action(&event(MouseEventKind::Moved, 0, 0)), None);
    }

    #[test]
    fn containment_excludes_the_far_edges() {
        let area = Rect::new(2, 3, 4, 2);
        assert!(contains(area, (2, 3)));
        assert!(contains(area, (5, 4)));
        assert!(!contains(area, (6, 4)));
        assert!(!contains(area, (5, 5)));
        assert!(!contains(area, (1, 3)));
    }
}
