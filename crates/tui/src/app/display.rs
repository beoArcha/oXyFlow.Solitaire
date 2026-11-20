//! Full screen: the contract the application implements, the facts the decision needs and how the
//! result reaches the player. The decision itself is the pure [`decide`].

use super::App;
use super::font::{ConsoleFont, NullConsoleFont};
use crate::settings::{Decision, TerminalKind, decide};

/// Switches the console window between full screen and its window. The application (`oxysol`)
/// implements it with the console API; the TUI never touches the console itself.
pub trait ConsoleDisplay {
    /// Switches full screen on or off. Returns whether the console obeyed; a refusal is not an
    /// error, the game goes on in the window it has.
    fn set_full_screen(&mut self, on: bool) -> bool;
}

/// A display that cannot switch anything.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullConsoleDisplay;

impl ConsoleDisplay for NullConsoleDisplay {
    fn set_full_screen(&mut self, _on: bool) -> bool {
        false
    }
}

/// What the application knows about the window and the command line, for the full-screen decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayFacts {
    /// Whether the game owns its console window: a double-click or `--new-window`.
    pub owns_window: bool,
    pub kind: TerminalKind,
    /// `--fullscreen` (`Some(true)`) or `--no-fullscreen` (`Some(false)`); it holds until the
    /// player changes the setting.
    pub flag: Option<bool>,
}

/// The display and the facts about it, as `Deps` carries them.
pub struct DisplayDeps {
    pub display: Box<dyn ConsoleDisplay>,
    pub facts: DisplayFacts,
    /// The console font behind Ctrl+Plus, Ctrl+Minus and Ctrl+0 (P3-16).
    pub font: Box<dyn ConsoleFont>,
}

impl DisplayDeps {
    /// A display that never changes anything, for a terminal the game does not own and for tests.
    pub fn none() -> Self {
        Self {
            display: Box::new(NullConsoleDisplay),
            facts: DisplayFacts {
                owns_window: false,
                kind: TerminalKind::Linux,
                flag: None,
            },
            font: Box::new(NullConsoleFont),
        }
    }
}

/// The console state of a running [`App`]: full screen and the font size.
pub(super) struct FullScreenState {
    pub(super) display: Box<dyn ConsoleDisplay>,
    pub(super) facts: DisplayFacts,
    pub(super) active: bool,
    pub(super) hint: Option<&'static str>,
    pub(super) hinted: bool,
    pub(super) font: Box<dyn ConsoleFont>,
}

impl FullScreenState {
    pub(super) fn new(deps: DisplayDeps) -> Self {
        Self {
            display: deps.display,
            facts: deps.facts,
            active: false,
            hint: None,
            hinted: false,
            font: deps.font,
        }
    }

    fn name_key(&mut self, key: &'static str) {
        if !self.hinted {
            self.hinted = true;
            self.hint = Some(key);
        }
    }
}

impl App {
    /// Brings the console in line with the setting: enters or leaves full screen, or names the
    /// terminal's own key once. A refused switch is a hint, never an error.
    pub(super) fn sync_full_screen(&mut self) {
        let state = &mut self.full_screen;
        let facts = state.facts;
        match decide(
            self.settings.full_screen,
            facts.flag,
            facts.owns_window,
            facts.kind,
        ) {
            Decision::Windowed => {
                if state.active {
                    state.active = !state.display.set_full_screen(false);
                }
            }
            Decision::Enter => {
                if !state.active {
                    state.active = state.display.set_full_screen(true);
                    if !state.active {
                        tracing::info!("the console refused full screen; naming its key instead");
                        state.name_key(facts.kind.hint_key());
                    }
                }
            }
            Decision::Hint => state.name_key(facts.kind.hint_key()),
        }
    }

    /// The player changed the setting: a flag was only for the start, so it stops applying.
    pub(super) fn full_screen_setting_changed(&mut self) {
        self.full_screen.facts.flag = None;
        self.sync_full_screen();
    }

    /// Puts the waiting hint, if any, into the `Now:` event slot of the board.
    pub(super) fn show_full_screen_hint(&mut self) {
        if let Some(key) = self.full_screen.hint.take() {
            self.board.announce_full_screen_hint(key);
        }
    }
}
