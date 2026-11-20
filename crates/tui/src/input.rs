//! Raw terminal input and the key table — the single source for key mapping, help and footer.

use ratatui::crossterm::event::{KeyEvent, MouseEvent};

mod table;

pub use table::{
    Binding, KEY_TABLE, KeySpec, action_for_key, binding_for, is_text_key, primary_key,
    primary_key_event,
};

/// One terminal event, already translated from the backend's event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

#[cfg(test)]
mod tests;
