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
            Effect::Restart => self.board.restart(self.now),
            Effect::Apply(settings) => self.apply_settings(settings),
            Effect::Open(modal) => self.open_modal(modal),
            Effect::Quit => {
                self.save_board();
                self.quit = true;
            }
        }
        self.sync_board();
    }

    pub(super) fn goto(&mut self, screen: Screen) {
        if self.screen == Screen::Board && screen != Screen::Board {
            self.save_board();
        }
        self.screen = screen;
        if screen == Screen::Board && self.modal.is_none() {
            self.board.resume_clock(self.now);
        } else {
            self.board.pause_clock(self.now);
        }
    }

    pub(super) fn open_modal(&mut self, modal: Modal) {
        self.board.pause_clock(self.now);
        self.board.set_covered(modal == Modal::Pause);
        self.modal = Some(modal);
    }

    pub(super) fn close_modal(&mut self) {
        self.modal = None;
        self.board.set_covered(false);
        if self.screen == Screen::Board {
            self.board.resume_clock(self.now);
        }
    }

    fn start_game(&mut self, variant: Variant, seed: Option<u64>) {
        self.abandon_saved();
        let seed = seed.unwrap_or_else(|| (self.seeds)());
        tracing::info!(?variant, seed, "new game dealt");
        let recorded = self.storage.start_game(variant, seed);
        let id = recorded.as_ref().ok().copied();
        self.board
            .begin(Session::new(variant, seed), id, Duration::ZERO, self.now);
        self.enter_board();
        self.note_write(recorded.map(drop), "the new game could not be recorded");
    }

    fn resume_game(&mut self) {
        let Some(saved) = self.saved.clone() else {
            return;
        };
        let (id, elapsed_ms) = (saved.id, saved.elapsed_ms);
        match Session::restore(saved.record) {
            Ok(session) => {
                let elapsed = Duration::from_millis(elapsed_ms);
                self.board.begin(session, Some(id), elapsed, self.now);
                self.enter_board();
            }
            Err(error) => {
                tracing::warn!(%error, "the saved game could not be restored; removing it");
                self.discard_unusable_save(Some((id, elapsed_ms)));
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
