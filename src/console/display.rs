//! Full screen of the console window: the kind of terminal the game runs in and the switch the TUI
//! asks for through `ConsoleDisplay`. Only the classic Windows console can be switched by the
//! game; every other terminal gets a hint that names its own key.

use std::io;

use oxysol_tui::app::ConsoleDisplay;
use oxysol_tui::caps::Platform;
use oxysol_tui::settings::TerminalKind;

/// The kind of terminal on `platform`. `env` reads an environment variable. On Windows,
/// `WT_SESSION` marks Windows Terminal; any other console is taken for the classic one.
pub fn terminal_kind(platform: Platform, env: &dyn Fn(&str) -> Option<String>) -> TerminalKind {
    match platform {
        Platform::Windows if env("WT_SESSION").is_some() => TerminalKind::WindowsTerminal,
        Platform::Windows => TerminalKind::ClassicConsole,
        Platform::MacOs => TerminalKind::MacOs,
        Platform::Linux => TerminalKind::Linux,
    }
}

/// The console window of this process. It leaves full screen again when dropped, if it entered it,
/// so a console that was not the game's own is returned as it was found.
#[derive(Debug, Default)]
pub struct ConsoleWindow {
    entered: bool,
}

impl ConsoleDisplay for ConsoleWindow {
    fn set_full_screen(&mut self, on: bool) -> bool {
        match switch(on) {
            Ok(()) => {
                self.entered = on;
                tracing::info!(on, "the console switched full screen");
                true
            }
            Err(error) => {
                tracing::info!(on, %error, "the console refused to switch full screen");
                false
            }
        }
    }
}

impl Drop for ConsoleWindow {
    fn drop(&mut self) {
        if self.entered {
            if let Err(error) = switch(false) {
                tracing::debug!(%error, "the console stayed in full screen at exit");
            }
        }
    }
}

/// Switches the console of this process to full screen or back to its window. A console that is not
/// a classic one, or output that is redirected, refuses with an error.
fn switch(full_screen: bool) -> io::Result<()> {
    platform::switch(full_screen)
}

/// The classic Windows console: `SetConsoleDisplayMode` where the system implements it, otherwise
/// the Alt+Enter keystroke posted to the console window, which is what conhost itself handles.
#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "the console and window calls have no safe wrapper in std and the crate avoids windows-sys"
)]
mod platform {
    use std::io;
    use std::os::windows::io::{AsRawHandle, RawHandle};
    use std::thread;
    use std::time::Duration;

    type Handle = RawHandle;

    #[repr(C)]
    struct Coord {
        x: i16,
        y: i16,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetConsoleDisplayMode(output: Handle, flags: u32, new_dimensions: *mut Coord) -> i32;
        fn GetConsoleDisplayMode(mode: *mut u32) -> i32;
        fn GetConsoleWindow() -> Handle;
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn PostMessageW(window: Handle, message: u32, w_param: usize, l_param: isize) -> i32;
    }

    const CONSOLE_FULLSCREEN_MODE: u32 = 1;
    const CONSOLE_WINDOWED_MODE: u32 = 2;
    const WM_SYSKEYDOWN: u32 = 0x0104;
    const WM_SYSKEYUP: u32 = 0x0105;
    const VK_RETURN: usize = 0x0D;
    const ENTER_SCAN_CODE: isize = 0x1C << 16;
    const ALT_CONTEXT: isize = 1 << 29;
    const KEY_UP_FLAGS: isize = 0b11 << 30;
    const POLLS: u32 = 25;
    const POLL: Duration = Duration::from_millis(20);

    pub fn switch(full_screen: bool) -> io::Result<()> {
        set_display_mode(full_screen).or_else(|refused| {
            tracing::debug!(%refused, "SetConsoleDisplayMode refused; trying Alt+Enter");
            toggle_with_alt_enter(full_screen)
        })
    }

    fn set_display_mode(full_screen: bool) -> io::Result<()> {
        let flags = if full_screen {
            CONSOLE_FULLSCREEN_MODE
        } else {
            CONSOLE_WINDOWED_MODE
        };
        let mut dimensions = Coord { x: 0, y: 0 };
        let output = io::stdout().as_raw_handle();
        // SAFETY: `output` is the live stdout handle, `dimensions` is a valid COORD that outlives the call.
        let accepted = unsafe { SetConsoleDisplayMode(output, flags, &mut dimensions) };
        if accepted == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn is_full_screen() -> io::Result<bool> {
        let mut mode = 0_u32;
        // SAFETY: `mode` is a valid u32 that outlives the call and is only written through the pointer.
        let read = unsafe { GetConsoleDisplayMode(&mut mode) };
        if read == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(mode != 0)
    }

    fn post_alt_enter() -> io::Result<()> {
        // SAFETY: GetConsoleWindow takes no arguments and returns this console's window or null.
        let window = unsafe { GetConsoleWindow() };
        if window.is_null() {
            return Err(io::ErrorKind::NotFound.into());
        }
        let down = ALT_CONTEXT | ENTER_SCAN_CODE | 1;
        let up = KEY_UP_FLAGS | down;
        // SAFETY: `window` is the handle just returned and the messages carry plain integers.
        let posted = unsafe {
            PostMessageW(window, WM_SYSKEYDOWN, VK_RETURN, down) != 0
                && PostMessageW(window, WM_SYSKEYUP, VK_RETURN, up) != 0
        };
        if posted {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn toggle_with_alt_enter(full_screen: bool) -> io::Result<()> {
        if is_full_screen()? == full_screen {
            return Ok(());
        }
        post_alt_enter()?;
        for _ in 0..POLLS {
            thread::sleep(POLL);
            if is_full_screen()? == full_screen {
                return Ok(());
            }
        }
        Err(io::ErrorKind::TimedOut.into())
    }
}

/// Without a console API the game cannot switch anything.
#[cfg(not(windows))]
mod platform {
    use std::io;

    pub fn switch(_full_screen: bool) -> io::Result<()> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_key: &str) -> Option<String> {
        None
    }

    fn windows_terminal(key: &str) -> Option<String> {
        (key == "WT_SESSION").then(|| "guid".to_owned())
    }

    #[test]
    fn windows_terminal_is_told_apart_by_its_session_variable() {
        assert_eq!(
            terminal_kind(Platform::Windows, &windows_terminal),
            TerminalKind::WindowsTerminal
        );
        assert_eq!(
            terminal_kind(Platform::Windows, &no_env),
            TerminalKind::ClassicConsole
        );
    }

    #[test]
    fn other_platforms_have_their_own_kind() {
        assert_eq!(
            terminal_kind(Platform::Linux, &windows_terminal),
            TerminalKind::Linux
        );
        assert_eq!(terminal_kind(Platform::MacOs, &no_env), TerminalKind::MacOs);
    }

    #[cfg(not(windows))]
    #[test]
    fn a_platform_without_a_console_api_refuses() {
        let mut window = ConsoleWindow::default();
        assert!(!window.set_full_screen(true));
        assert!(!window.entered);
    }
}
