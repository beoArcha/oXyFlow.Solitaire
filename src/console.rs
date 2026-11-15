//! Console window ownership and the wait that keeps a double-clicked window open after an error.

use std::io::{self, IsTerminal, Write};

use oxysol_tui::i18n;

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
}
