//! The clipboard contract. The application (`oxysol`) implements it; the TUI only asks to copy
//! text and announces the result.

use thiserror::Error;

/// Why a copy failed. The text is never lost: the screen that asked also shows it.
#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("no clipboard is available")]
    Unavailable,
    #[error("the text could not be sent to the clipboard")]
    Io(#[from] std::io::Error),
}

/// Copies text to the system clipboard. Implementations must never block for long and never
/// fail the game.
pub trait Clipboard {
    /// Puts `text` on the clipboard.
    fn copy(&mut self, text: &str) -> Result<(), ClipboardError>;
}

/// A clipboard that is never there: every copy fails with [`ClipboardError::Unavailable`].
#[derive(Debug, Default, Clone, Copy)]
pub struct NullClipboard;

impl Clipboard for NullClipboard {
    fn copy(&mut self, _text: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_null_clipboard_always_fails() {
        assert!(matches!(
            NullClipboard.copy("42"),
            Err(ClipboardError::Unavailable)
        ));
    }
}
