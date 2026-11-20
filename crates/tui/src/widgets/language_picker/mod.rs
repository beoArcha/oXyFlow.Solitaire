//! The language selector of the main menu: `◄ [flag] ► ▼` in the top-left corner (`< EN > v` in
//! ASCII or 16-color mode, where the code replaces the flag). Left and Right, or a click on an
//! arrow, switch to the neighbouring language at once; `▼`, Down and Enter open a list of all
//! languages, chosen by arrows and Enter or by a click, closed by Esc or a click outside. A tooltip
//! names the language in itself while the pointer is over the selector or it has the focus.
//!
//! The picker only reports the language the player chose; the caller applies and saves it as the
//! same setting as the Language row of the settings screen.

mod draw;

use ratatui::crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::action::Action;
use crate::i18n::Locale;
use crate::mouse::{contains, left_press, wheel_action};
use crate::screens::focus::step;

/// What the picker made of a mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointed {
    /// Not the picker's: the event goes on to the rest of the screen.
    Ignored,
    /// The picker used the event; nothing else should react to it.
    Handled,
    /// The selector was clicked and takes the keyboard focus.
    Focus,
    /// The player chose this language.
    Choose(Locale),
}

#[derive(Debug, Default, Clone)]
struct Spots {
    body: Rect,
    left: Rect,
    right: Rect,
    down: Rect,
    list: Rect,
    entries: Vec<Rect>,
}

/// State of the selector: whether its list is open and where it was drawn last.
#[derive(Debug, Default)]
pub struct LanguagePicker {
    open: Option<usize>,
    spots: Spots,
}

fn index_of(locale: Locale) -> usize {
    Locale::ALL
        .iter()
        .position(|candidate| *candidate == locale)
        .unwrap_or(0)
}

fn shifted(current: Locale, forward: bool) -> Locale {
    let next = step(index_of(current), Locale::ALL.len(), forward);
    Locale::ALL.get(next).copied().unwrap_or(current)
}

impl LanguagePicker {
    /// A closed selector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the list of languages is open.
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Closes the list without choosing.
    pub fn close(&mut self) {
        self.open = None;
    }

    fn open_at(&mut self, current: Locale) {
        self.open = Some(index_of(current));
    }

    /// Handles a key while the selector has the focus or its list is open. Returns the language
    /// the player chose, if any. While the list is open it takes every key.
    pub fn on_action(&mut self, action: Action, current: Locale) -> Option<Locale> {
        if let Some(index) = self.open {
            return self.on_open_action(index, action);
        }
        match action {
            Action::Left => Some(shifted(current, false)),
            Action::Right => Some(shifted(current, true)),
            Action::Down | Action::Select => {
                self.open_at(current);
                None
            }
            _ => None,
        }
    }

    fn on_open_action(&mut self, index: usize, action: Action) -> Option<Locale> {
        let len = Locale::ALL.len();
        match action {
            Action::Up => self.open = Some(step(index, len, false)),
            Action::Down => self.open = Some(step(index, len, true)),
            Action::Select => {
                self.open = None;
                return Locale::ALL.get(index).copied();
            }
            Action::Back => self.open = None,
            _ => {}
        }
        None
    }

    /// Handles a mouse event. While the list is open it takes every event.
    pub fn on_mouse(&mut self, event: &MouseEvent, current: Locale) -> Pointed {
        if let Some(index) = self.open {
            return self.on_open_mouse(index, event);
        }
        let Some(cell) = left_press(event) else {
            return Pointed::Ignored;
        };
        let spots = &self.spots;
        if contains(spots.left, cell) {
            Pointed::Choose(shifted(current, false))
        } else if contains(spots.right, cell) {
            Pointed::Choose(shifted(current, true))
        } else if contains(spots.down, cell) {
            self.open_at(current);
            Pointed::Focus
        } else if contains(spots.body, cell) {
            Pointed::Focus
        } else {
            Pointed::Ignored
        }
    }

    fn on_open_mouse(&mut self, index: usize, event: &MouseEvent) -> Pointed {
        if let Some(action) = wheel_action(event) {
            self.on_open_action(index, action);
            return Pointed::Handled;
        }
        let cell = (event.column, event.row);
        let over = self
            .spots
            .entries
            .iter()
            .position(|entry| contains(*entry, cell));
        if event.kind == MouseEventKind::Moved {
            self.open = over.or(self.open);
        } else if left_press(event).is_some() {
            return self.on_open_press(over, cell);
        }
        Pointed::Handled
    }

    fn on_open_press(&mut self, over: Option<usize>, cell: (u16, u16)) -> Pointed {
        match over.and_then(|index| Locale::ALL.get(index).copied()) {
            Some(locale) => {
                self.open = None;
                Pointed::Choose(locale)
            }
            None => {
                if !contains(self.spots.list, cell) {
                    self.open = None;
                }
                Pointed::Handled
            }
        }
    }
}

#[cfg(test)]
mod tests;
