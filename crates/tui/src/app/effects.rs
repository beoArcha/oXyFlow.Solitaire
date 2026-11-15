//! Applying an [`Effect`]: navigation, the modal layer, starting and resuming games, settings.

use std::time::Duration;

use oxysol_core::{Session, Variant};

use super::{App, Modal, Screen};
use crate::screens::Effect;
use crate::settings::Settings;

impl App {
    pub(crate) fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::None => {}
            Effect::Goto(screen) => self.goto(screen),
            Effect::Start { variant, seed } => self.start_game(variant, seed),
            Effect::Resume => self.resume_game(),
            Effect::Apply(settings) => self.apply_settings(settings),
            Effect::Open(modal) => self.open_modal(modal),
            Effect::Quit => self.quit = true,
        }
    }

    fn goto(&mut self, screen: Screen) {
        self.screen = screen;
        if screen == Screen::Board {
            self.board.resume_clock(self.now);
        } else {
            self.board.pause_clock(self.now);
        }
    }

    fn open_modal(&mut self, modal: Modal) {
        self.board.pause_clock(self.now);
        self.modal = Some(modal);
    }

    pub(super) fn close_modal(&mut self) {
        self.modal = None;
        if self.screen == Screen::Board {
            self.board.resume_clock(self.now);
        }
    }

    fn start_game(&mut self, variant: Variant, seed: Option<u64>) {
        let seed = seed.unwrap_or_else(|| (self.seeds)());
        let id = match self.storage.start_game(variant, seed) {
            Ok(id) => Some(id),
            Err(error) => {
                tracing::warn!(%error, ?variant, seed, "the new game could not be recorded");
                None
            }
        };
        self.board
            .begin(Session::new(variant, seed), id, Duration::ZERO, self.now);
        self.goto(Screen::Board);
        if id.is_none() {
            self.open_modal(Modal::alert("common.error.storage"));
        }
    }

    fn resume_game(&mut self) {
        let Some(saved) = self.saved.take() else {
            return;
        };
        match Session::restore(saved.record) {
            Ok(session) => {
                let elapsed = Duration::from_millis(saved.elapsed_ms);
                self.board.begin(session, Some(saved.id), elapsed, self.now);
                self.goto(Screen::Board);
            }
            Err(error) => {
                tracing::warn!(%error, "the saved game could not be restored");
                self.open_modal(Modal::alert("common.error.resume"));
            }
        }
    }

    fn apply_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.refresh_view();
        if let Err(error) = self.storage.save_settings(&self.settings) {
            tracing::warn!(%error, "settings could not be saved; they hold for this session");
            self.open_modal(Modal::alert("common.error.storage"));
        }
    }
}
