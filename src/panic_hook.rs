//! The panic hook (card P4-02). Order: restore the terminal, tell the player where the details
//! are, write one diagnostic record through the log handle opened at start.

use std::backtrace::{Backtrace, BacktraceStatus};
use std::fs::File;
use std::io::{self, Write};
use std::panic::{self, PanicHookInfo};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use oxysol_tui::i18n;

/// Environment variable that makes the binary panic on purpose, so the hook can be tested in a
/// child process. Only the exact value `1` counts; no player sets it.
pub const TEST_PANIC_VARIABLE: &str = "OXYSOL_TEST_PANIC";

/// Installs the hook. `diagnostics` is the log handle from [`crate::logging::start`]; without it
/// the stderr message still appears and the record is skipped.
pub fn install(log_path: PathBuf, diagnostics: Option<File>) {
    panic::set_hook(Box::new(move |info| {
        oxysol_tui::restore_terminal();
        let location = location_of(info);
        let text = i18n::tf(
            "cli.panic",
            &[
                ("location", location.as_str()),
                ("path", &log_path.display().to_string()),
            ],
        );
        best_effort(io::stderr(), &format!("{text}\n"));
        if let Some(file) = diagnostics.as_ref() {
            let backtrace = captured_backtrace();
            let line = format_record(unix_seconds(), &location, message_of(info), backtrace);
            best_effort(file, &line);
        }
    }));
}

/// Panics on purpose when `OXYSOL_TEST_PANIC` is exactly `1`. Called after the hook is installed
/// and before the terminal starts.
#[allow(
    clippy::panic,
    reason = "the deliberate test trigger for the hook; reachable only with OXYSOL_TEST_PANIC=1"
)]
pub fn panic_if_requested() {
    if std::env::var(TEST_PANIC_VARIABLE).is_ok_and(|value| value == "1") {
        panic::panic_any("panic requested by OXYSOL_TEST_PANIC");
    }
}

/// Writes `text` and drops the outcome. A panic hook has nowhere to report a failed write, and a
/// second panic here would hide the first one. This is the only ignored write result in the crate.
#[allow(
    clippy::let_underscore_must_use,
    reason = "a panic hook cannot report a failed write and must not panic again"
)]
fn best_effort(mut out: impl Write, text: &str) {
    let _ = out.write_all(text.as_bytes());
}

fn location_of(info: &PanicHookInfo<'_>) -> String {
    info.location().map_or_else(
        || "an unknown location".to_owned(),
        |place| format!("{}:{}", place.file(), place.line()),
    )
}

fn message_of<'a>(info: &'a PanicHookInfo<'_>) -> &'a str {
    let payload = info.payload();
    if let Some(text) = payload.downcast_ref::<&str>() {
        text
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.as_str()
    } else {
        "a panic without a message"
    }
}

fn captured_backtrace() -> Option<String> {
    let backtrace = Backtrace::capture();
    (backtrace.status() == BacktraceStatus::Captured).then(|| backtrace.to_string())
}

/// One record in the shape of the `tracing` lines, with the backtrace after it when captured.
fn format_record(seconds: u64, location: &str, message: &str, backtrace: Option<String>) -> String {
    let mut line = format!("{seconds} ERROR oxysol::critical: panic at {location}: {message}\n");
    if let Some(trace) = backtrace {
        line.push_str(&trace);
        line.push('\n');
    }
    line
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_names_the_level_target_location_and_message() {
        let line = format_record(7, "src/x.rs:3", "boom", None);
        assert_eq!(
            line,
            "7 ERROR oxysol::critical: panic at src/x.rs:3: boom\n"
        );
    }

    #[test]
    fn a_captured_backtrace_follows_the_record() {
        let line = format_record(7, "src/x.rs:3", "boom", Some("frame".to_owned()));
        assert!(line.ends_with("frame\n"), "{line}");
        assert_eq!(line.lines().count(), 2);
    }
}
