//! Console window ownership and the wait that keeps a double-clicked window open after an error.

use std::io::{self, IsTerminal, Write};

use oxysol_tui::i18n;

pub mod card_font;
pub mod display;
pub mod font;

#[cfg(test)]
mod face_tests;

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

/// Clamps a requested window size `(columns, rows)` to `largest`, or returns `None` when
/// either dimension is zero.
pub fn fit_window(requested: (u16, u16), largest: (u16, u16)) -> Option<(u16, u16)> {
    let columns = requested.0.min(largest.0);
    let rows = requested.1.min(largest.1);
    if columns == 0 || rows == 0 {
        return None;
    }
    Some((columns, rows))
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
#[allow(
    unsafe_code,
    reason = "Win32 console sizing APIs have no safe wrappers in std and the crate avoids windows-sys"
)]
mod win {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct Coord {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct SmallRect {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct ConsoleScreenBufferInfo {
        size: Coord,
        cursor_position: Coord,
        attributes: u16,
        window: SmallRect,
        maximum_window_size: Coord,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            file_name: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *const c_void,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: *mut c_void,
        ) -> *mut c_void;
        fn CloseHandle(object: *mut c_void) -> i32;
        fn GetConsoleMode(console_handle: *mut c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(console_handle: *mut c_void, mode: u32) -> i32;
        fn WriteConsoleA(
            console_output: *mut c_void,
            buffer: *const u8,
            chars_to_write: u32,
            chars_written: *mut u32,
            reserved: *mut c_void,
        ) -> i32;
        fn GetConsoleScreenBufferInfo(
            console_output: *mut c_void,
            info: *mut ConsoleScreenBufferInfo,
        ) -> i32;
        fn GetLargestConsoleWindowSize(console_output: *mut c_void) -> Coord;
        fn SetConsoleScreenBufferSize(console_output: *mut c_void, size: Coord) -> i32;
        fn SetConsoleWindowInfo(
            console_output: *mut c_void,
            absolute: i32,
            console_window: *const SmallRect,
        ) -> i32;
    }

    const GENERIC_READ_WRITE: u32 = 0xC000_0000;
    const FILE_SHARE_READ_WRITE: u32 = 0x0000_0003;
    const OPEN_EXISTING: u32 = 3;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
    const CONOUT: [u16; 8] = [
        b'C' as u16,
        b'O' as u16,
        b'N' as u16,
        b'O' as u16,
        b'U' as u16,
        b'T' as u16,
        b'$' as u16,
        0,
    ];

    struct OwnedHandle(*mut c_void);

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            // SAFETY: `self.0` is a valid handle returned by `CreateFileW`.
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    pub(super) struct ConsoleHandle {
        stdout: OwnedHandle,
        buffer: Coord,
        pub(super) current: (u16, u16),
        pub(super) largest: (u16, u16),
    }

    fn u16_pair(x: i32, y: i32) -> Option<(u16, u16)> {
        if x <= 0 || y <= 0 {
            return None;
        }
        Some((u16::try_from(x).ok()?, u16::try_from(y).ok()?))
    }

    fn window_size_of(info: &ConsoleScreenBufferInfo) -> Option<(u16, u16)> {
        u16_pair(
            i32::from(info.window.right) - i32::from(info.window.left) + 1,
            i32::from(info.window.bottom) - i32::from(info.window.top) + 1,
        )
    }

    pub(super) fn query_console() -> Option<ConsoleHandle> {
        // SAFETY: `CONOUT` is null-terminated UTF-16 `"CONOUT$"` and all flags are valid constants.
        let raw = unsafe {
            CreateFileW(
                CONOUT.as_ptr(),
                GENERIC_READ_WRITE,
                FILE_SHARE_READ_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };
        if raw.is_null() || raw == (-1_isize as *mut c_void) {
            tracing::debug!("the active console output handle is not available");
            return None;
        }
        let stdout = OwnedHandle(raw);
        let mut info = ConsoleScreenBufferInfo::default();
        // SAFETY: stdout.0 is valid and info points to stack memory.
        if unsafe { GetConsoleScreenBufferInfo(stdout.0, &mut info) } == 0 {
            tracing::debug!("failed to read console screen buffer info");
            return None;
        }
        let current = window_size_of(&info)?;
        // SAFETY: stdout.0 is a valid console handle.
        let max = unsafe { GetLargestConsoleWindowSize(stdout.0) };
        let largest = u16_pair(i32::from(max.x), i32::from(max.y))?;
        Some(ConsoleHandle {
            stdout,
            buffer: info.size,
            current,
            largest,
        })
    }

    fn set_buffer(stdout: *mut c_void, size: Coord) -> bool {
        // SAFETY: stdout is a valid console handle and size holds positive dimensions.
        let ok = unsafe { SetConsoleScreenBufferSize(stdout, size) } != 0;
        if !ok {
            tracing::debug!(
                x = size.x,
                y = size.y,
                "failed to set console screen buffer size"
            );
        }
        ok
    }

    fn resize_with_vt(stdout: *mut c_void, target: (u16, u16)) -> bool {
        let mut mode = 0_u32;
        // SAFETY: stdout is a valid console handle and mode is stack-allocated.
        if unsafe { GetConsoleMode(stdout, &mut mode) } == 0 {
            return false;
        }
        if mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
            // SAFETY: stdout is a valid console handle and mode adds VT processing.
            let _ = unsafe { SetConsoleMode(stdout, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) };
        }
        let seq = format!("\x1b[8;{};{}t", target.1, target.0);
        let Ok(len) = u32::try_from(seq.len()) else {
            return false;
        };
        let mut written = 0_u32;
        // SAFETY: stdout is a valid console handle and seq.as_ptr() is valid for len bytes.
        if unsafe {
            WriteConsoleA(
                stdout,
                seq.as_ptr(),
                len,
                &mut written,
                std::ptr::null_mut(),
            )
        } == 0
        {
            return false;
        }
        let mut info = ConsoleScreenBufferInfo::default();
        // SAFETY: stdout is valid and info points to stack memory.
        if unsafe { GetConsoleScreenBufferInfo(stdout, &mut info) } == 0 {
            return false;
        }
        window_size_of(&info) == Some(target)
    }

    pub(super) fn apply_window_size(
        handle: &ConsoleHandle,
        target: (u16, u16),
        exact_buffer: bool,
    ) -> bool {
        let (Ok(target_x), Ok(target_y)) = (i16::try_from(target.0), i16::try_from(target.1))
        else {
            return false;
        };
        let safe_buffer = Coord {
            x: handle.buffer.x.max(target_x),
            y: handle.buffer.y.max(target_y),
        };
        if safe_buffer != handle.buffer && !set_buffer(handle.stdout.0, safe_buffer) {
            return false;
        }
        let rect = SmallRect {
            left: 0,
            top: 0,
            right: target_x - 1,
            bottom: target_y - 1,
        };
        // SAFETY: stdout.0 is a valid handle and rect fits within safe_buffer.
        if unsafe { SetConsoleWindowInfo(handle.stdout.0, 1, &rect) } == 0
            && !resize_with_vt(handle.stdout.0, target)
        {
            let error = std::io::Error::last_os_error();
            tracing::debug!(target_x, target_y, %error, "failed to resize console window");
            return false;
        }
        let exact = Coord {
            x: target_x,
            y: target_y,
        };
        if exact_buffer && exact != safe_buffer && !set_buffer(handle.stdout.0, exact) {
            let _ = resize_with_vt(handle.stdout.0, target);
        }
        true
    }
}

/// Widens an owned console window to at least 130 columns and 30 rows if the screen allows it.
#[cfg(windows)]
pub fn widen_owned_window() {
    let Some(handle) = win::query_console() else {
        return;
    };
    let Some(target) = widened(handle.current, handle.largest) else {
        return;
    };
    let _ = win::apply_window_size(&handle, target, false);
}

/// Resizing an owned console window is only implemented on Windows.
#[cfg(not(windows))]
pub fn widen_owned_window() {}

/// Requests `columns` by `rows` (clamped to the largest window that fits) for an owned classic
/// console window. Returns whether the resize was applied.
#[cfg(windows)]
pub fn set_owned_window_size(columns: u16, rows: u16) -> bool {
    if !owns_window() {
        return false;
    }
    let Some(handle) = win::query_console() else {
        return false;
    };
    let Some(target) = fit_window((columns, rows), handle.largest) else {
        return false;
    };
    win::apply_window_size(&handle, target, true)
}

/// Resizing an owned console window is only implemented on Windows.
#[cfg(not(windows))]
pub fn set_owned_window_size(_columns: u16, _rows: u16) -> bool {
    false
}

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

    #[test]
    fn default_console_target_is_130_columns() {
        assert_eq!(TARGET_COLUMNS, 130);
        assert_eq!(TARGET_ROWS, 30);
    }

    #[test]
    fn fit_window_clamps_to_largest_and_rejects_zero() {
        assert_eq!(fit_window((130, 40), (200, 60)), Some((130, 40)));
        assert_eq!(fit_window((260, 80), (220, 55)), Some((220, 55)));
        assert_eq!(fit_window((0, 40), (200, 60)), None);
        assert_eq!(fit_window((130, 40), (200, 0)), None);
    }
}
