//! The docked help window in the application: where it sits, which of the window and the board
//! has the keys, and whether the board clock runs. The layout and the keys themselves are in
//! `widgets::overlay`; this file only asks them and applies the answers.

use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;

use super::{App, Screen};
use crate::action::Action;
use crate::mouse::{self, left_press};
use crate::screens::Effect;
use crate::widgets::overlay;

impl App {
    /// The docked window's rectangle and the screen beside it, when the open modal is a help
    /// window that docks at the current size.
    pub(super) fn dock_split(&self) -> Option<(Rect, Rect)> {
        let modal = self.modal.as_ref()?;
        if !overlay::dock::dock_view::is_dockable(modal) {
            return None;
        }
        overlay::dock::split(self.screen_rect(), self.settings.help_dock)
    }

    /// Whether the open modal takes the keys: a centred modal always does, a docked one only
    /// while it has the focus. The board takes them otherwise.
    pub(super) fn modal_takes_keys(&self) -> bool {
        self.modal.is_some() && (self.dock_split().is_none() || self.dock.focused)
    }

    /// Gives the keys to the docked window (`true`) or back to the board, and keeps the clock in
    /// step: it stops while a modal takes the keys and runs while the board does.
    pub(super) fn focus_dock(&mut self, focused: bool) {
        self.dock.focused = focused;
        self.sync_clock();
    }

    /// Stops the board clock while a modal takes the keys, and runs it while the board has them.
    pub(super) fn sync_clock(&mut self) {
        if self.modal_takes_keys() {
            self.board.pause_clock(self.now);
        } else if self.screen == Screen::Board {
            self.board.resume_clock(self.now);
        }
    }

    /// Moves the help window to the next side (centre, right, left) and remembers the choice.
    pub(super) fn cycle_dock(&mut self) {
        self.settings.help_dock = self.settings.help_dock.next();
        self.sync_clock();
        self.persist_settings();
    }

    /// Routes a key while a modal is open: `Some(effect)` when the dock controls or the modal take
    /// it, `None` when the board does (a docked window without the focus). From the board, `?`
    /// closes a docked window, and so does Esc when no card is picked up.
    pub(super) fn dock_route(&mut self, key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
        self.modal.as_ref()?;
        if let Some(effect) = self.dock_key(action) {
            return Some(effect);
        }
        if self.modal_takes_keys() {
            return Some(self.on_modal_key(key, action));
        }
        let leaves = action == Some(Action::Help)
            || (action == Some(Action::Back) && !self.board.carrying());
        if leaves {
            self.close_modal();
            return Some(Effect::None);
        }
        None
    }

    /// The keys that work whenever a help window is open: `d` cycles its side and F6 moves the
    /// focus between the board and a docked window. `None` leaves the key to the rest.
    pub(super) fn dock_key(&mut self, action: Option<Action>) -> Option<Effect> {
        match action? {
            Action::Dock
                if self
                    .modal
                    .as_ref()
                    .is_some_and(overlay::dock::dock_view::is_dockable) =>
            {
                self.cycle_dock();
                Some(Effect::None)
            }
            Action::DockFocus if self.dock_split().is_some() => {
                let focused = !self.dock.focused;
                self.focus_dock(focused);
                Some(Effect::None)
            }
            _ => None,
        }
    }

    /// Whether `cell` lies on the placement button of the open help window, docked or centred.
    pub(super) fn dock_button_hit(&self, cell: (u16, u16)) -> bool {
        let Some(modal) = self.modal.as_ref() else {
            return false;
        };
        let frame = match self.dock_split() {
            Some((rect, _)) => Some(rect),
            None => overlay::dock::dock_view::centred_frame(modal, self.screen_rect()),
        };
        frame.is_some_and(|frame| overlay::dock::button_hit(frame, self.settings.help_dock, cell))
    }

    /// Handles a mouse event that lands on the docked window at `rect`: a click gives it the keys.
    pub(super) fn on_docked_mouse(&mut self, event: &MouseEvent, rect: Rect) -> Effect {
        if !mouse::contains(rect, (event.column, event.row)) {
            return Effect::None;
        }
        if !self.dock.focused {
            self.focus_dock(true);
        }
        let rules = overlay::rules_of(&self.board);
        let outcome = match self.modal.as_mut() {
            Some(modal) => overlay::dock::handle_mouse(&mut self.dock, modal, rect, rules, event),
            None => None,
        };
        self.sync_quick();
        self.finish_modal(outcome)
    }

    /// Routes a mouse event while a help window is open. `Some(effect)` when the window takes it:
    /// a centred window takes every event, a docked one the events over it. `None` lets the event
    /// through to the board, which stays under a docked window without the focus.
    pub(super) fn dock_mouse(&mut self, event: &MouseEvent) -> Option<Effect> {
        self.modal.as_ref()?;
        if left_press(event).is_some_and(|cell| self.dock_button_hit(cell)) {
            self.cycle_dock();
            return Some(Effect::None);
        }
        let Some((rect, _)) = self.dock_split() else {
            return Some(self.on_modal_mouse(event));
        };
        if mouse::contains(rect, (event.column, event.row)) {
            return Some(self.on_docked_mouse(event, rect));
        }
        if left_press(event).is_some() && self.dock.focused {
            self.focus_dock(false);
        }
        None
    }

    /// Handles a key while a modal takes the keys: a docked window gets the keys of its own
    /// handler, a centred modal those of the overlay.
    pub(super) fn on_modal_key(&mut self, key: &KeyEvent, action: Option<Action>) -> Effect {
        let rules = overlay::rules_of(&self.board);
        let docked = self.dock_split().map(|(rect, _)| rect);
        let outcome = match (self.modal.as_mut(), docked) {
            (Some(modal), Some(rect)) => {
                overlay::dock::handle_key(&mut self.dock, modal, rect, rules, key, action)
            }
            (Some(modal), None) => overlay::handle_for(modal, key, action, rules),
            (None, _) => None,
        };
        self.sync_quick();
        self.finish_modal(outcome)
    }
}
