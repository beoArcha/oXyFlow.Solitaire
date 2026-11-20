//! The console font of the classic Windows console: the contract the application implements and the
//! keys that ask for it. Ctrl+Plus, Ctrl+Minus and Ctrl+0 are the zoom keys of nearly every other
//! terminal, which handles them itself and never sends them on; only a console without a zoom of
//! its own delivers them, and then the game changes the font size. The console calls themselves are
//! made by the application (`oxysol`), never by the TUI.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// What a font key asks of the console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStep {
    /// A larger font.
    Larger,
    /// A smaller font.
    Smaller,
    /// The font size the game started with.
    Reset,
}

/// Changes the font size of the console window. The application implements it for the classic
/// Windows console; every other terminal uses [`NullConsoleFont`].
pub trait ConsoleFont {
    /// Applies `step`. Returns whether the font changed; a refusal or a limit is not an error.
    fn step(&mut self, step: FontStep) -> bool;
}

/// A console whose font the game cannot change.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullConsoleFont;

impl ConsoleFont for NullConsoleFont {
    fn step(&mut self, _step: FontStep) -> bool {
        false
    }
}

/// The font step a key press asks for: Ctrl+Plus (also reported as Ctrl+`=`, the same key without
/// Shift), Ctrl+Minus and Ctrl+0. Shift is ignored, as it is for every character key.
pub fn font_step_for_key(key: &KeyEvent) -> Option<FontStep> {
    if key.kind == KeyEventKind::Release
        || key.modifiers - KeyModifiers::SHIFT != KeyModifiers::CONTROL
    {
        return None;
    }
    match key.code {
        KeyCode::Char('+' | '=') => Some(FontStep::Larger),
        KeyCode::Char('-') => Some(FontStep::Smaller),
        KeyCode::Char('0') => Some(FontStep::Reset),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn ctrl_plus_minus_and_zero_are_the_font_keys() {
        assert_eq!(
            font_step_for_key(&ctrl(KeyCode::Char('+'))),
            Some(FontStep::Larger)
        );
        assert_eq!(
            font_step_for_key(&ctrl(KeyCode::Char('='))),
            Some(FontStep::Larger)
        );
        assert_eq!(
            font_step_for_key(&KeyEvent::new(
                KeyCode::Char('='),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            Some(FontStep::Larger)
        );
        assert_eq!(
            font_step_for_key(&ctrl(KeyCode::Char('-'))),
            Some(FontStep::Smaller)
        );
        assert_eq!(
            font_step_for_key(&ctrl(KeyCode::Char('0'))),
            Some(FontStep::Reset)
        );
    }

    #[test]
    fn the_keys_without_ctrl_and_other_keys_are_not_font_keys() {
        for code in ['+', '=', '-', '0', ']', '[', 'a'] {
            let plain = KeyEvent::new(KeyCode::Char(code), KeyModifiers::NONE);
            assert_eq!(font_step_for_key(&plain), None, "{code}");
        }
        assert_eq!(font_step_for_key(&ctrl(KeyCode::Char('c'))), None);
        assert_eq!(font_step_for_key(&ctrl(KeyCode::Up)), None);
        let alt = KeyEvent::new(
            KeyCode::Char('0'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert_eq!(font_step_for_key(&alt), None);
    }

    #[test]
    fn a_key_release_is_not_a_font_key() {
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('0'),
            KeyModifiers::CONTROL,
            KeyEventKind::Release,
        );
        assert_eq!(font_step_for_key(&release), None);
    }

    #[test]
    fn the_null_console_never_changes() {
        let mut console = NullConsoleFont;
        for step in [FontStep::Larger, FontStep::Smaller, FontStep::Reset] {
            assert!(!console.step(step));
        }
    }
}
