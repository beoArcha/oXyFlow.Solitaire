//! Terminal setup, the event loop and restoring the terminal on every exit path.

use std::io;
use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::cursor::{Hide, Show};
use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use thiserror::Error;

use crate::app::{App, Deps};
use crate::input::Input;

/// How long the loop waits for input before it sends a tick (and redraws the clock).
const TICK: Duration = Duration::from_millis(200);

/// An operational failure of the terminal: raw mode refused, output failed.
#[derive(Debug, Error)]
pub enum TuiError {
    #[error("terminal input or output failed")]
    Io(#[from] io::Error),
}

/// Puts the terminal into game mode on creation and restores it when dropped, so every exit path
/// (normal, error, unwinding panic) leaves the terminal as it was found.
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let guard = Self;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture, Hide)?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn report(step: &str, result: io::Result<()>) {
    if let Err(error) = result {
        tracing::warn!(step, %error, "terminal restore step failed");
    }
}

/// Leaves the alternate screen, shows the cursor, releases the mouse and leaves raw mode. Every
/// step is tried even if an earlier one failed. Also called from the panic hook.
pub fn restore_terminal() {
    let mut out = io::stdout();
    report("show cursor", execute!(out, Show));
    report("disable mouse capture", execute!(out, DisableMouseCapture));
    report(
        "leave alternate screen",
        execute!(out, LeaveAlternateScreen),
    );
    report("disable raw mode", disable_raw_mode());
}

pub(crate) fn translate(event: Event) -> Input {
    match event {
        Event::Key(key) => Input::Key(key),
        Event::Mouse(mouse) => Input::Mouse(mouse),
        Event::Resize(width, height) => Input::Resize(width, height),
        _ => Input::Tick,
    }
}

fn read_input(timeout: Duration) -> io::Result<Input> {
    if event::poll(timeout)? {
        Ok(translate(event::read()?))
    } else {
        Ok(Input::Tick)
    }
}

/// The loop: draw, wait for one input, handle it, until the application wants to quit.
pub(crate) fn drive(
    app: &mut App,
    mut draw: impl FnMut(&mut App) -> io::Result<()>,
    mut next: impl FnMut() -> io::Result<Input>,
) -> io::Result<()> {
    while !app.should_quit() {
        draw(app)?;
        let input = next()?;
        app.handle(input, Instant::now());
    }
    Ok(())
}

/// Runs the application on the real terminal until the player quits.
pub fn run(deps: Deps) -> Result<(), TuiError> {
    let mut app = App::new(deps);
    let _guard = TerminalGuard::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    drive(
        &mut app,
        |app| terminal.draw(|frame| app.draw(frame)).map(|_| ()),
        || read_input(TICK),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
