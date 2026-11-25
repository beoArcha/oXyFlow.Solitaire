//! The application shell: screens, the modal layer, settings and the services behind `Deps`.
//! Screens never change shared state themselves; they return an [`Effect`] and `App` applies it.

use std::time::Instant;

#[cfg(test)]
use crate::input::Input;
#[cfg(test)]
use crate::screens::Effect;
#[cfg(test)]
use ratatui::crossterm::event::KeyEventKind;

use self::deals::Check;
use self::display::FullScreenState;
use self::priority::PriorityState;
use self::search::Search;
use crate::audio::{Audio, StyleInfo};
use crate::caps::TerminalCaps;
use crate::card_style::{CardStyles, DEFAULT_BACK, DEFAULT_FRONT};
use crate::clipboard::Clipboard;
use crate::glyphs::Glyphs;
use crate::i18n;
use crate::screens::board::BoardScreen;
use crate::screens::game_select::GameSelect;
use crate::screens::main_menu::MainMenu;
use crate::screens::settings::SettingsScreen;
use crate::screens::stats::StatsScreen;
use crate::settings::{Aids, DEFAULT_SOUND_STYLE, Settings};
use crate::solver::Solver;
use crate::storage::{AssistCounters, GameId, SavedGame, Storage, StorageError};
use crate::theme::Theme;
use crate::widgets::footer::FooterMap;
use crate::widgets::overlay;

pub use deps::Deps;
pub use display::{ConsoleDisplay, DisplayDeps, DisplayFacts, NullConsoleDisplay};
pub use font::{
    CardFontState, ConsoleFont, FontFacts, FontStep, NullConsoleFont, font_step_for_key,
};
pub use modal::Modal;
pub use priority::{NullPriority, PriorityControl, PriorityDeps, PriorityOutcome};
pub use search::{SEED_CAP, SearchCommand, SearchProgress};

/// The screen behind the modal layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    GameSelect,
    Board,
    Settings,
    Stats,
}

macro_rules! view_ctx {
    ($app:expr) => {
        ViewCtx {
            theme: &$app.theme,
            glyphs: &$app.glyphs,
            settings: if $app.screen == Screen::Board {
                &$app.running
            } else {
                &$app.settings
            },
            caps: &$app.caps,
            audio_status: $app.audio.status(),
            sound_styles: &$app.sound_styles,
            saved: $app.saved_summary(),
            pointer: $app.pointer,
            now: $app.now,
            font: $app.font_facts(),
        }
    };
}

/// The whole application state.
pub struct App {
    storage: Box<dyn Storage>,
    audio: Box<dyn Audio>,
    clipboard: Box<dyn Clipboard>,
    seeds: Box<dyn FnMut() -> u64>,
    solver: Box<dyn Solver>,
    full_screen: FullScreenState,
    priority: PriorityState,
    search: Option<Search>,
    check: Option<Check>,
    next_query: u64,
    caps: TerminalCaps,
    sound_styles: Vec<StyleInfo>,
    settings: Settings,
    interface_baseline: Option<Settings>,
    quick: Aids,
    running: Settings,
    theme: Theme,
    glyphs: Glyphs,
    saved: Option<SavedGame>,
    write_failed: bool,
    assist_written: Option<(GameId, AssistCounters)>,
    settings_write_failed: bool,
    screen: Screen,
    modal: Option<Modal>,
    dock: overlay::dock::DockState,
    menu: MainMenu,
    select: GameSelect,
    settings_screen: SettingsScreen,
    stats: StatsScreen,
    board: BoardScreen,
    modal_footer: FooterMap,
    area: Option<(u16, u16)>,
    pointer: Option<(u16, u16)>,
    now: Instant,
    quit: bool,
}

impl App {
    /// Builds the application: loads the settings and the saved game from storage, applies the
    /// settings to the theme, the language and the audio. A storage failure opens an alert and
    /// the defaults are used.
    pub fn new(deps: Deps) -> Self {
        let Deps {
            storage,
            audio,
            clipboard,
            caps,
            seeds,
            start_seed,
            sound_styles,
            solver,
            display,
            version,
        } = deps;
        let (display, priority) = display.split_priority();
        let settings = Settings::for_caps(&caps);
        let mut app = Self {
            storage,
            audio,
            clipboard,
            seeds,
            solver,
            full_screen: FullScreenState::new(display),
            priority: PriorityState::new(priority),
            search: None,
            check: None,
            next_query: 0,
            caps,
            sound_styles,
            theme: Theme::new(settings.theme, settings.color_depth(&caps))
                .with_color_vision(settings.color_vision),
            glyphs: Glyphs::styled(
                settings.drawn_glyph_mode(false),
                &settings.front_style,
                &settings.back_style,
            ),
            quick: settings.aids(),
            running: settings.clone(),
            interface_baseline: None,
            settings,
            saved: None,
            write_failed: false,
            assist_written: None,
            settings_write_failed: false,
            screen: Screen::MainMenu,
            modal: None,
            dock: overlay::dock::DockState::default(),
            menu: MainMenu::start(start_seed, version),
            select: GameSelect::new(),
            settings_screen: SettingsScreen::new(),
            stats: StatsScreen::new(),
            board: BoardScreen::new(),
            area: None,
            pointer: None,
            modal_footer: FooterMap::default(),
            now: Instant::now(),
            quit: false,
        };
        app.load_stored_state();
        app.fit_sound_style();
        app.fit_card_styles();
        app.sync_console_look();
        app.sync_full_screen();
        app.sync_priority();
        app.reset_quick();
        app.refresh_view();
        app
    }

    /// A saved sound style that no longer exists falls back to the default style.
    fn fit_sound_style(&mut self) {
        let known = self
            .sound_styles
            .iter()
            .any(|style| style.id == self.settings.sound_style);
        if !known {
            self.settings.sound_style = DEFAULT_SOUND_STYLE.to_owned();
        }
    }

    /// A saved front or back style that no longer exists falls back to the default style.
    fn fit_card_styles(&mut self) {
        let styles = CardStyles::embedded();
        if !styles.has_front(&self.settings.front_style) {
            self.settings.front_style = DEFAULT_FRONT.to_owned();
        }
        if !styles.has_back(&self.settings.back_style) {
            self.settings.back_style = DEFAULT_BACK.to_owned();
        }
    }

    fn load_stored_state(&mut self) {
        match self.storage.load_settings() {
            Ok(Some(stored)) => {
                self.settings = Settings {
                    locale: stored.locale.or_source(),
                    ..stored
                };
            }
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(%error, "stored settings could not be loaded; using defaults");
                self.modal = Some(Modal::alert("common.error.settings_load"));
            }
        }
        match self.storage.load_game() {
            Ok(saved) => self.saved = saved,
            Err(error) => {
                tracing::warn!(%error, "saved game could not be loaded");
                self.modal
                    .get_or_insert(Modal::alert("common.error.resume"));
                if matches!(
                    error,
                    StorageError::Corrupt { .. } | StorageError::Incompatible { .. }
                ) {
                    self.discard_unusable_save(None);
                }
            }
        }
    }

    fn refresh_view(&mut self) {
        self.theme = Theme::new(self.settings.theme, self.settings.color_depth(&self.caps))
            .with_color_vision(self.settings.color_vision);
        self.glyphs = Glyphs::styled(
            self.settings
                .drawn_glyph_mode(self.console_shows_card_font()),
            &self.settings.front_style,
            &self.settings.back_style,
        );
        i18n::set_locale(self.settings.locale);
        self.audio.configure(&self.settings.audio());
    }

    /// The screen behind the modal layer.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// The open modal, if any.
    pub fn modal(&self) -> Option<&Modal> {
        self.modal.as_ref()
    }

    /// The settings in effect.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The board screen, with the running session and the clock.
    pub fn board(&self) -> &BoardScreen {
        &self.board
    }

    #[cfg(test)]
    pub(crate) fn board_mut(&mut self) -> &mut BoardScreen {
        &mut self.board
    }

    /// Whether the player asked to quit; the event loop ends when this is true.
    pub fn should_quit(&self) -> bool {
        self.quit
    }
}

mod deals;
mod deps;
mod display;
mod dock;
mod draw;
mod effects;
mod event;
mod finish;
mod font;
mod interface;
mod modal;
mod persist;
mod priority;
mod quick;
mod records;
mod search;

#[cfg(test)]
mod card_font_tests;
#[cfg(test)]
mod display_tests;
#[cfg(test)]
mod font_tests;

#[cfg(test)]
mod priority_tests;
#[cfg(test)]
mod records_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod chosen_deal_tests;

#[cfg(test)]
mod deals_tests;

#[cfg(test)]
mod effect_tests;

#[cfg(test)]
mod path_tests;

#[cfg(test)]
mod persist_tests;

#[cfg(test)]
mod dock_tests;
