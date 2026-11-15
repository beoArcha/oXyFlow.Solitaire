//! The system clipboard behind [`oxysol_tui::Clipboard`] (decision T-27). The native clipboard
//! (`arboard`) is tried first; when it is missing or the session is remote, the text goes to the
//! terminal as an OSC 52 escape sequence, which most terminals turn into a clipboard write.
//!
//! A terminal gives no answer to OSC 52, so a successful write to it means "sent", not "stored".

use std::io::{self, Write};

use oxysol_tui::{Clipboard, ClipboardError};

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard Base64 with padding.
fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let group = chunk.iter().enumerate().fold(0_u32, |acc, (index, byte)| {
            acc | u32::from(*byte) << (16 - 8 * index)
        });
        for position in 0..4 {
            if position <= chunk.len() {
                let index = usize::try_from(group >> (18 - 6 * position) & 63).unwrap_or(0);
                out.push(char::from(BASE64[index]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The escape sequence that asks the terminal to put `text` on the system clipboard.
pub fn osc52(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64(text.as_bytes()))
}

/// Native clipboard first, OSC 52 as the fallback.
pub struct SystemClipboard {
    native: Option<arboard::Clipboard>,
    remote: bool,
    terminal: Box<dyn Write>,
}

impl SystemClipboard {
    /// A clipboard that falls back to the terminal on standard output. In a `remote` session
    /// (SSH) the native clipboard would be the server's, so only the terminal is used.
    pub fn new(remote: bool) -> Self {
        Self::with_terminal(remote, Box::new(io::stdout()))
    }

    /// Like [`SystemClipboard::new`] with another terminal stream.
    pub fn with_terminal(remote: bool, terminal: Box<dyn Write>) -> Self {
        Self {
            native: None,
            remote,
            terminal,
        }
    }

    fn copy_native(&mut self, text: &str) -> bool {
        if self.native.is_none() {
            match arboard::Clipboard::new() {
                Ok(clipboard) => self.native = Some(clipboard),
                Err(error) => {
                    tracing::debug!(%error, "no native clipboard");
                    return false;
                }
            }
        }
        let Some(clipboard) = self.native.as_mut() else {
            return false;
        };
        match clipboard.set_text(text.to_owned()) {
            Ok(()) => true,
            Err(error) => {
                tracing::debug!(%error, "the native clipboard refused the text");
                false
            }
        }
    }

    fn copy_through_terminal(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.terminal.write_all(osc52(text).as_bytes())?;
        self.terminal.flush()?;
        Ok(())
    }
}

impl Clipboard for SystemClipboard {
    fn copy(&mut self, text: &str) -> Result<(), ClipboardError> {
        if !self.remote && self.copy_native(text) {
            return Ok(());
        }
        self.copy_through_terminal(text)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    struct Shared(Rc<RefCell<Vec<u8>>>);

    impl Write for Shared {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct Broken;

    impl Write for Broken {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn base64_matches_the_standard_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64(b"12345"), "MTIzNDU=");
    }

    #[test]
    fn osc52_wraps_the_base64_text_for_the_clipboard_selection() {
        assert_eq!(osc52("12345"), "\u{1b}]52;c;MTIzNDU=\u{7}");
    }

    #[test]
    fn a_remote_session_copies_through_the_terminal_only() {
        let sent = Rc::new(RefCell::new(Vec::new()));
        let mut clipboard =
            SystemClipboard::with_terminal(true, Box::new(Shared(Rc::clone(&sent))));
        assert!(clipboard.copy("12345").is_ok());
        assert_eq!(sent.borrow().as_slice(), osc52("12345").as_bytes());
    }

    #[test]
    fn a_terminal_that_cannot_be_written_is_reported_with_its_cause() {
        let mut clipboard = SystemClipboard::with_terminal(true, Box::new(Broken));
        let error = clipboard.copy("1");
        assert!(matches!(error, Err(ClipboardError::Io(ref cause))
            if cause.kind() == io::ErrorKind::BrokenPipe));
    }
}
