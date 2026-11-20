//! Console window ownership and the wait that keeps a double-clicked window open after an error.

use std::io::{self, IsTerminal, Write};

use oxysol_tui::i18n;

pub mod card_font;
pub mod display;
pub mod font;

/// Minimum width for an owned console window, fitting the guide panel.
pub const TARGET_COLUMNS: u16 = 130;

/// Minimum height for an owned console window.
pub const TARGET_ROWS: u16 = 30;

/// Returns the widened window size `(columns, rows)`, or `None` when no change is needed or possible.
pub fn widened(current: (u16, u16), largest: (u16, u16)) -> Option<(u16, u16)> {
    if largest.0 < TARGET_COLUMNS {
        return None;
    }
    if current.0 >= TARGET_COLUMNS && current.1 >= TARGET_ROWS {
        return None;
    }
    let width = current.0.max(TARGET_COLUMNS);
    let height = current.1.max(TARGET_ROWS);
    if width > largest.0 || height > largest.1 {
        return None;
    }
    Some((width, height))
}

/// Decides whether the process should wait for Enter before it exits.
///
/// Only a failed run, in a console that this process created, with an interactive stdin, waits.
/// A successful run and a run from an existing terminal or a pipe never wait.
pub fn should_pause(failed: bool, owns_window: bool, stdin_is_terminal: bool) -> bool {
    failed && owns_window && stdin_is_terminal
}

/// Whether this process is the only one attached to its console, so the console was created
/// for it, for example by a double-click in Explorer.
#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "GetConsoleProcessList has no safe wrapper in std and the crate avoids windows-sys"
)]
pub fn owns_window() -> bool {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetConsoleProcessList(process_list: *mut u32, count: u32) -> u32;
    }
    let mut processes = [0_u32; 2];
    // SAFETY: the buffer holds exactly two u32 and its length is passed; the count may exceed it.
    let attached = unsafe { GetConsoleProcessList(processes.as_mut_ptr(), 2) };
    attached == 1
}

/// Without a console API there is no window to own.
#[cfg(not(windows))]
pub fn owns_window() -> bool {
    false
}

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Coord {
    x: i16,
    y: i16,
}

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SmallRect {
    left: i16,
    top: i16,
    right: i16,
    bottom: i16,
}

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct ConsoleScreenBufferInfo {
    size: Coord,
    cursor_position: Coord,
    attributes: u16,
    window: SmallRect,
    maximum_window_size: Coord,
}

/// Widens an owned console window to at least 130 columns and 30 rows if the screen allows it.
#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "Win32 console sizing APIs have no safe wrappers in std and the crate avoids windows-sys"
)]
pub fn widen_owned_window() {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(std_handle: u32) -> *mut std::ffi::c_void;
        fn GetConsoleScreenBufferInfo(
            console_output: *mut std::ffi::c_void,
            info: *mut ConsoleScreenBufferInfo,
        ) -> i32;
        fn GetLargestConsoleWindowSize(console_output: *mut std::ffi::c_void) -> Coord;
        fn SetConsoleScreenBufferSize(console_output: *mut std::ffi::c_void, size: Coord) -> i32;
        fn SetConsoleWindowInfo(
            console_output: *mut std::ffi::c_void,
            absolute: i32,
            console_window: *const SmallRect,
        ) -> i32;
    }

    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;

    // SAFETY: GetStdHandle is called with standard constant STD_OUTPUT_HANDLE.
    let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if stdout.is_null() || stdout == (-1_isize as *mut std::ffi::c_void) {
        tracing::debug!("the standard output handle is not available");
        return;
    }

    let mut info = ConsoleScreenBufferInfo::default();
    // SAFETY: stdout is checked for validity and info points to valid stack memory.
    let info_ok = unsafe { GetConsoleScreenBufferInfo(stdout, &mut info) };
    if info_ok == 0 {
        tracing::debug!("failed to read console screen buffer info");
        return;
    }

    let current_cols = i32::from(info.window.right) - i32::from(info.window.left) + 1;
    let current_rows = i32::from(info.window.bottom) - i32::from(info.window.top) + 1;
    if current_cols <= 0 || current_rows <= 0 {
        tracing::debug!("invalid current console window dimensions");
        return;
    }
    let Ok(cols) = u16::try_from(current_cols) else {
        tracing::debug!("console columns exceed u16");
        return;
    };
    let Ok(rows) = u16::try_from(current_rows) else {
        tracing::debug!("console rows exceed u16");
        return;
    };

    // SAFETY: stdout is a valid console handle.
    let largest = unsafe { GetLargestConsoleWindowSize(stdout) };
    if largest.x <= 0 || largest.y <= 0 {
        tracing::debug!("failed to read largest console window size");
        return;
    }
    let Ok(max_cols) = u16::try_from(largest.x) else {
        tracing::debug!("largest columns exceed u16");
        return;
    };
    let Ok(max_rows) = u16::try_from(largest.y) else {
        tracing::debug!("largest rows exceed u16");
        return;
    };

    let Some((new_cols, new_rows)) = widened((cols, rows), (max_cols, max_rows)) else {
        return;
    };

    let Ok(target_x) = i16::try_from(new_cols) else {
        tracing::debug!("target columns exceed i16");
        return;
    };
    let Ok(target_y) = i16::try_from(new_rows) else {
        tracing::debug!("target rows exceed i16");
        return;
    };

    let target_buffer = Coord {
        x: info.size.x.max(target_x),
        y: info.size.y.max(target_y),
    };
    // SAFETY: stdout is a valid handle and target buffer dimensions are non-negative.
    let buffer_ok = unsafe { SetConsoleScreenBufferSize(stdout, target_buffer) };
    if buffer_ok == 0 {
        tracing::debug!("failed to set console screen buffer size");
        return;
    }

    let target_window = SmallRect {
        left: 0,
        top: 0,
        right: target_x - 1,
        bottom: target_y - 1,
    };
    // SAFETY: stdout is a valid handle and target window fits within the updated screen buffer.
    let window_ok = unsafe { SetConsoleWindowInfo(stdout, 1, &target_window) };
    if window_ok == 0 {
        tracing::debug!("failed to set console window info");
    }
}

/// Resizing an owned console window is only implemented on Windows.
#[cfg(not(windows))]
pub fn widen_owned_window() {}

/// Prints the localized prompt to stderr and waits for one line from stdin.
pub fn wait_for_enter() {
    let prompt = i18n::t("cli.press_enter");
    if let Err(error) = writeln!(io::stderr(), "{prompt}") {
        tracing::debug!(%error, "the wait prompt cannot be written");
        return;
    }
    let mut line = String::new();
    if let Err(error) = io::stdin().read_line(&mut line) {
        tracing::debug!(%error, "the wait for Enter ended by a read error");
    }
}

/// Whether stdin is an interactive terminal, which a double-clicked console window provides.
pub fn stdin_is_terminal() -> bool {
    io::stdin().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_failed_run_in_an_owned_interactive_console_pauses() {
        assert!(should_pause(true, true, true));
        assert!(!should_pause(false, true, true));
        assert!(!should_pause(true, false, true));
        assert!(!should_pause(true, true, false));
        assert!(!should_pause(false, false, false));
    }

    #[test]
    fn widened_increases_width_to_target_and_preserves_larger_width() {
        assert_eq!(widened((120, 30), (200, 60)), Some((130, 30)));
        assert_eq!(widened((100, 30), (200, 60)), Some((130, 30)));
        assert_eq!(widened((140, 20), (200, 60)), Some((140, 30)));
    }

    #[test]
    fn widened_ensures_minimum_rows_and_keeps_larger_height() {
        assert_eq!(widened((120, 25), (200, 60)), Some((130, 30)));
        assert_eq!(widened((120, 45), (200, 60)), Some((130, 45)));
    }

    #[test]
    fn widened_returns_none_when_both_targets_already_met() {
        assert_eq!(widened((130, 30), (200, 60)), None);
        assert_eq!(widened((130, 40), (200, 60)), None);
        assert_eq!(widened((150, 50), (200, 60)), None);
    }

    #[test]
    fn widened_returns_none_when_largest_cannot_fit_target_columns() {
        assert_eq!(widened((120, 30), (125, 60)), None);
        assert_eq!(widened((100, 30), (129, 60)), None);
    }

    #[test]
    fn widened_returns_none_when_largest_cannot_fit_target_rows() {
        assert_eq!(widened((120, 25), (200, 28)), None);
        assert_eq!(widened((120, 45), (200, 40)), None);
    }
}
