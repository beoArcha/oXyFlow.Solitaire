//! Full screen: the contract the application implements, the facts the decision needs and how the
//! result reaches the player. The decision itself is the pure [`decide`].

use super::App;
use super::font::{CardFontState, ConsoleFont, FontFacts, NullConsoleFont};
use super::priority::PriorityDeps;
use crate::screens::Effect;
use crate::settings::{Decision, GlyphMode, Settings, TerminalKind, decide};
use crate::widgets::overlay::GraphicsCheck;

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
    /// The process priority control and the `--priority` flag (P4-13).
    pub priority: PriorityDeps,
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
            priority: PriorityDeps::none(),
        }
    }

    /// The priority part, and the rest with a priority that does nothing.
    pub(super) fn split_priority(mut self) -> (Self, PriorityDeps) {
        let priority = std::mem::replace(&mut self.priority, PriorityDeps::none());
        (self, priority)
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
    pub(super) card_font: CardFontState,
    pub(super) face: CardFontState,
    pub(super) size_asked: u8,
}

impl FullScreenState {
    pub(super) fn new(deps: DisplayDeps) -> Self {
        let facts = deps.facts;
        Self {
            display: deps.display,
            facts,
            active: false,
            hint: None,
            hinted: false,
            font: deps.font,
            card_font: Self::initial_state(facts.kind),
            face: Self::initial_state(facts.kind),
            size_asked: 0,
        }
    }

    fn initial_state(kind: TerminalKind) -> CardFontState {
        if kind == TerminalKind::ClassicConsole {
            CardFontState::Off
        } else {
            CardFontState::Unavailable
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
            Decision::Hint => {
                if !state.hinted {
                    tracing::info!(
                        kind = ?facts.kind,
                        owns_window = facts.owns_window,
                        setting = ?self.settings.full_screen,
                        flag = ?facts.flag,
                        "full screen is left to the terminal; naming its key instead"
                    );
                }
                state.name_key(facts.kind.hint_key());
            }
        }
    }

    /// The player changed the setting: a flag was only for the start, so it stops applying.
    pub(super) fn full_screen_setting_changed(&mut self) {
        self.full_screen.facts.flag = None;
        self.sync_full_screen();
    }

    /// Brings the console font in line with the option "Use the oXySol font in this console": asks
    /// for the card font when it is on and sets the old font back when it is off. A refusal is a
    /// state the Settings row shows, never an error.
    pub(super) fn sync_card_font(&mut self) {
        let wanted = self.settings.console_font;
        let state = &mut self.full_screen;
        state.card_font = match (wanted, state.card_font) {
            (true, CardFontState::Active) => CardFontState::Active,
            (true, _) => state.font.use_card_font(true),
            (false, CardFontState::Active) => state.font.use_card_font(false),
            (false, CardFontState::NotRegistered | CardFontState::Refused) => CardFontState::Off,
            (false, other) => other,
        };
    }

    /// Brings the console font in line with the settings: the card font first, which owns the face
    /// while it is in use, then the chosen face and the chosen size. One console object does all
    /// three, so the font the game found stays the one it puts back at exit.
    pub(super) fn sync_console_look(&mut self) {
        self.sync_card_font();
        self.sync_face();
        self.sync_size();
    }

    fn sync_face(&mut self) {
        let wanted = self.settings.console_face.name();
        let state = &mut self.full_screen;
        state.face = match (wanted, state.face, state.card_font) {
            (_, CardFontState::Unavailable, _) => CardFontState::Unavailable,
            (_, _, CardFontState::Active) => CardFontState::Off,
            (None, CardFontState::Off, _) => CardFontState::Off,
            (None, _, _) => state.font.use_face(None),
            (Some(name), _, _) => state.font.use_face(Some(name)),
        };
    }

    fn sync_size(&mut self) {
        let wanted = self.settings.console_font_size;
        let state = &mut self.full_screen;
        if state.card_font == CardFontState::Unavailable || wanted == state.size_asked {
            return;
        }
        let height = (wanted > 0).then_some(u16::from(wanted));
        if state.font.use_size(height) {
            state.size_asked = wanted;
        }
    }

    /// The option was changed: switches the console font and, when the card font is in use now,
    /// the symbols to it (the font is known to be there, so no graphics check is needed).
    pub(super) fn console_font_setting_changed(&mut self) {
        self.sync_console_look();
        if self.settings.console_font && self.full_screen.card_font == CardFontState::Active {
            self.settings.glyph_mode = GlyphMode::Font;
        }
    }

    /// What the screens know about the console font.
    pub(super) fn font_facts(&self) -> FontFacts {
        FontFacts {
            terminal: self.full_screen.facts.kind,
            card_font: self.full_screen.card_font,
            face: self.full_screen.face,
            cell_px: self.full_screen.font.cell_size(),
            priority: self.priority.outcome,
        }
    }

    /// Whether the card font is in use because the console shows it.
    pub(super) fn console_shows_card_font(&self) -> bool {
        self.full_screen.card_font == CardFontState::Active
    }

    /// Whether the settings ask for the Font glyphs although nothing yet shows that the font is
    /// there: the player must pass the graphics check first.
    pub(super) fn needs_graphics_check(&self, next: &Settings) -> bool {
        next.glyph_mode == GlyphMode::Font
            && self.settings.glyph_mode != GlyphMode::Font
            && !next.font_confirmed
            && !self.console_shows_card_font()
    }

    /// Opens the graphics check: samples in the card font and the question. "Yes" switches the
    /// glyph mode to Font and remembers the confirmation.
    pub(super) fn open_graphics_check(&mut self) {
        let confirmed = Settings {
            glyph_mode: GlyphMode::Font,
            font_confirmed: true,
            ..self.settings.clone()
        };
        let windows_terminal = self.full_screen.facts.kind == TerminalKind::WindowsTerminal;
        let check = GraphicsCheck::new(&self.settings, windows_terminal, Effect::Apply(confirmed));
        self.open_modal(super::Modal::GraphicsCheck(check));
    }

    /// Puts the waiting hint, if any, into the `Now:` event slot of the board.
    pub(super) fn show_full_screen_hint(&mut self) {
        if let Some(key) = self.full_screen.hint.take() {
            self.board.announce_full_screen_hint(key);
        }
    }
}
