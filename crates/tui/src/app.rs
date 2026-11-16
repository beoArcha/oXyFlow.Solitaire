//! The application shell: screens, the modal layer, settings and the services behind `Deps`.
//! Screens never change shared state themselves; they return an [`Effect`] and `App` applies it.

use std::time::Instant;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, KeyEventKind, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::Block;

use crate::action::Action;
use crate::audio::{Audio, StyleInfo};
use crate::caps::TerminalCaps;
use crate::clipboard::Clipboard;
use crate::glyphs::Glyphs;
use crate::i18n;
use crate::input::{Input, action_for_key, is_text_key};
use crate::screens::board::BoardScreen;
use crate::screens::game_select::GameSelect;
use crate::screens::main_menu::MainMenu;
use crate::screens::settings::SettingsScreen;
use crate::screens::stats::StatsScreen;
use crate::screens::{Effect, ViewCtx};
use crate::settings::{DEFAULT_SOUND_STYLE, Settings};
use crate::storage::{SavedGame, Storage, StorageError};
use crate::theme::Theme;
use crate::widgets::overlay;
use crate::widgets::too_small::{self, is_too_small};

pub use deps::Deps;
pub use modal::Modal;

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
            settings: &$app.settings,
            caps: &$app.caps,
            audio_status: $app.audio.status(),
            sound_styles: &$app.sound_styles,
            has_saved_game: $app.saved.is_some(),
            now: $app.now,
        }
    };
}

/// The whole application state.
pub struct App {
    storage: Box<dyn Storage>,
    audio: Box<dyn Audio>,
    clipboard: Box<dyn Clipboard>,
    seeds: Box<dyn FnMut() -> u64>,
    caps: TerminalCaps,
    sound_styles: Vec<StyleInfo>,
    settings: Settings,
    theme: Theme,
    glyphs: Glyphs,
    saved: Option<SavedGame>,
    write_failed: bool,
    screen: Screen,
    modal: Option<Modal>,
    menu: MainMenu,
    select: GameSelect,
    settings_screen: SettingsScreen,
    stats: StatsScreen,
    board: BoardScreen,
    area: Option<(u16, u16)>,
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
        } = deps;
        let settings = Settings::for_caps(&caps);
        let mut app = Self {
            storage,
            audio,
            clipboard,
            seeds,
            caps,
            sound_styles,
            theme: Theme::new(settings.theme, settings.color_depth(&caps)),
            glyphs: Glyphs::new(settings.glyph_mode),
            settings,
            saved: None,
            write_failed: false,
            screen: Screen::MainMenu,
            modal: None,
            menu: start_seed.map_or_else(MainMenu::new, MainMenu::for_seed),
            select: GameSelect::new(),
            settings_screen: SettingsScreen::new(),
            stats: StatsScreen::new(),
            board: BoardScreen::new(),
            area: None,
            now: Instant::now(),
            quit: false,
        };
        app.load_stored_state();
        app.fit_sound_style();
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

    fn load_stored_state(&mut self) {
        match self.storage.load_settings() {
            Ok(Some(stored)) => self.settings = stored,
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
        self.theme = Theme::new(self.settings.theme, self.settings.color_depth(&self.caps));
        self.glyphs = Glyphs::new(self.settings.glyph_mode);
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

    fn too_small(&self) -> bool {
        self.area
            .is_some_and(|(width, height)| is_too_small(Rect::new(0, 0, width, height)))
    }

    /// Handles one input event. `now` drives the clock; the application never reads the time.
    pub fn handle(&mut self, input: Input, now: Instant) {
        self.now = now;
        let effect = match input {
            Input::Key(key) => self.on_key(key),
            Input::Mouse(event) => self.on_mouse(&event),
            Input::Resize(width, height) => {
                self.area = Some((width, height));
                self.board.resize(width, height);
                Effect::None
            }
            Input::Tick => Effect::None,
        };
        self.apply(effect);
        self.play_board_cues();
    }

    fn play_board_cues(&mut self) {
        for cue in self.board.take_cues() {
            self.audio.play(cue);
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> Effect {
        if key.kind == KeyEventKind::Release {
            return Effect::None;
        }
        let action = action_for_key(&key);
        if action == Some(Action::ForceQuit) {
            return Effect::Quit;
        }
        if self.too_small() {
            return if action == Some(Action::Quit) {
                Effect::Quit
            } else {
                Effect::None
            };
        }
        if self.modal.is_some() {
            return self.on_modal_key(&key, action);
        }
        if !self.types_text(&key) {
            match action {
                Some(Action::Help) => return Effect::Open(Modal::Help),
                Some(Action::Rules) => return Effect::HowToPlay,
                _ => {}
            }
        }
        let ctx = view_ctx!(self);
        match self.screen {
            Screen::MainMenu => self.menu.handle(&key, action, &ctx),
            Screen::GameSelect => self.select.handle(&key, action, &ctx),
            Screen::Board => self.board.handle(&key, action, &ctx),
            Screen::Settings => self.settings_screen.handle(&key, action, &ctx),
            Screen::Stats => self.stats.handle(&key, action),
        }
    }

    /// Whether the active screen is typing into a text field.
    fn captures_text(&self) -> bool {
        match self.screen {
            Screen::MainMenu => self.menu.captures_text(),
            Screen::GameSelect | Screen::Board | Screen::Settings | Screen::Stats => false,
        }
    }

    /// Whether `key` is typed text for the active screen, so that it is not a global shortcut.
    fn types_text(&self, key: &KeyEvent) -> bool {
        self.captures_text() && is_text_key(key)
    }

    fn on_modal_key(&mut self, key: &KeyEvent, action: Option<Action>) -> Effect {
        let outcome = self
            .modal
            .as_mut()
            .and_then(|modal| overlay::handle(modal, key, action));
        self.finish_modal(outcome)
    }

    fn on_modal_mouse(&mut self, event: &MouseEvent) -> Effect {
        let area = self.screen_rect();
        let outcome = self
            .modal
            .as_mut()
            .and_then(|modal| overlay::handle_mouse(modal, area, &self.board, self.now, event));
        self.finish_modal(outcome)
    }

    fn finish_modal(&mut self, outcome: Option<Effect>) -> Effect {
        match outcome {
            Some(effect) => {
                self.close_modal();
                effect
            }
            None => Effect::None,
        }
    }

    fn screen_rect(&self) -> Rect {
        self.area.map_or(Rect::default(), |(width, height)| {
            Rect::new(0, 0, width, height)
        })
    }

    fn on_mouse(&mut self, event: &MouseEvent) -> Effect {
        if self.too_small() {
            return Effect::None;
        }
        if self.modal.is_some() {
            return self.on_modal_mouse(event);
        }
        let ctx = view_ctx!(self);
        match self.screen {
            Screen::MainMenu => self.menu.handle_mouse(event, &ctx),
            Screen::GameSelect => self.select.handle_mouse(event, &ctx),
            Screen::Board => self.board.handle_mouse(event, &ctx),
            Screen::Settings => self.settings_screen.handle_mouse(event, &ctx),
            Screen::Stats => self.stats.handle_mouse(event),
        }
    }

    /// Draws the current screen, then the modal on top. Below the minimum size only the notice
    /// is drawn.
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.area = Some((area.width, area.height));
        let ctx = view_ctx!(self);
        frame.render_widget(Block::new().style(ctx.theme.base()), area);
        if is_too_small(area) {
            too_small::draw(frame, area, &ctx);
            return;
        }
        match self.screen {
            Screen::MainMenu => self.menu.draw(frame, area, &ctx),
            Screen::GameSelect => self.select.draw(frame, area, &ctx),
            Screen::Board => self.board.draw(frame, area, &ctx),
            Screen::Settings => self.settings_screen.draw(frame, area, &ctx),
            Screen::Stats => self.stats.draw(frame, area, &ctx),
        }
        if let Some(modal) = &self.modal {
            overlay::draw(frame, area, modal, &self.board, &ctx);
        }
    }
}

mod deps;
mod effects;
mod modal;
mod persist;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod effect_tests;

#[cfg(test)]
mod persist_tests;
