//! Applying an [`Effect`]: navigation, the modal layer, starting and resuming games, settings.

use std::time::Duration;

use oxysol_core::{Session, Variant};

use super::deals::Origin;
use super::{App, Modal, Screen};
use crate::action::Action;
use crate::input::primary_key_event;
use crate::screens::Effect;
use crate::screens::board::Start;
use crate::settings::Settings;
use crate::solver::{DealCheck, Proof};
use crate::widgets::overlay::{HowToPlay, Rules};

impl App {
    pub(crate) fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::None => {}
            Effect::Goto(screen) => self.goto(screen),
            Effect::Start {
                variant,
                seed,
                undo_allowed,
            } => self.start_game(variant, seed, undo_allowed),
            Effect::SearchStep(command) => self.search_step(command),
            Effect::Resume => self.resume_game(),
            Effect::SameSeed => self.same_seed(),
            Effect::Restart => self.restart_game(),
            Effect::Apply(settings) => self.apply_settings(settings),
            Effect::Open(modal) => self.open_modal(modal),
            Effect::Quick(aids) => self.set_quick(aids),
            Effect::SaveAids => self.save_quick(),
            Effect::Board(action) => self.run_board_action(action),
            Effect::HowToPlay => self.open_how_to_play(),
            Effect::CopySeed(seed) => self.copy_seed(seed),
            Effect::Quit => {
                self.save_board();
                self.quit = true;
            }
        }
        self.sync_board();
    }

    fn run_board_action(&mut self, action: Action) {
        if self.screen != Screen::Board {
            return;
        }
        if let Some(key) = primary_key_event(action) {
            let effect = self.run(key, Some(action));
            self.apply(effect);
        }
    }

    pub(super) fn goto(&mut self, screen: Screen) {
        if self.screen == Screen::Board && screen != Screen::Board {
            self.save_board();
        }
        self.screen = screen;
        if screen == Screen::Stats {
            self.refresh_stats();
        }
        if screen == Screen::Board && self.modal.is_none() {
            self.board.resume_clock(self.now);
        } else {
            self.board.pause_clock(self.now);
        }
    }

    fn refresh_stats(&mut self) {
        match self.storage.history() {
            Ok(records) => self.stats.show(records),
            Err(error) => {
                tracing::warn!(%error, "the game history could not be read");
                self.stats.show_failure();
            }
        }
    }

    /// Copies a seed to the clipboard and announces the result on the statistics screen. A failure
    /// is logged here, once per attempt, and the seed stays visible as text.
    pub(crate) fn copy_seed(&mut self, seed: u64) {
        let copied = match self.clipboard.copy(&seed.to_string()) {
            Ok(()) => true,
            Err(error) => {
                tracing::warn!(%error, "the seed could not be copied");
                false
            }
        };
        self.stats.announce_copy(seed, copied);
        self.board.announce_copy(seed, copied);
    }

    pub(super) fn open_modal(&mut self, modal: Modal) {
        let modal = self.with_personal_best(modal);
        self.board.pause_clock(self.now);
        self.board.set_covered(modal == Modal::Pause);
        self.modal = Some(modal);
    }

    /// Deals the game that was just played again from its seed, through the new-game path.
    fn same_seed(&mut self) {
        let Some(session) = self.board.session() else {
            return;
        };
        let (variant, seed) = (session.variant(), session.seed());
        let undo_allowed = self.board.undo_allowed();
        self.start_game(variant, Some(seed), undo_allowed);
    }

    fn open_how_to_play(&mut self) {
        let rules = self
            .board
            .session()
            .map_or(HowToPlay::new(Rules::Klondike), |session| {
                HowToPlay::for_variant(session.variant())
            });
        self.open_modal(Modal::HowToPlay(rules));
    }

    pub(super) fn close_modal(&mut self) {
        self.modal = None;
        self.board.set_covered(false);
        if self.screen == Screen::Board {
            self.board.resume_clock(self.now);
        }
    }

    fn start_game(&mut self, variant: Variant, seed: Option<u64>, undo_allowed: bool) {
        match seed {
            Some(seed) => self.begin_game(variant, seed, undo_allowed, Origin::Chosen),
            None if self.settings.winnable_only => self.begin_search(variant, undo_allowed),
            None => {
                let seed = (self.seeds)();
                self.begin_game(variant, seed, undo_allowed, Origin::Fresh);
            }
        }
    }

    pub(super) fn begin_game(
        &mut self,
        variant: Variant,
        seed: u64,
        undo_allowed: bool,
        origin: Origin,
    ) {
        self.abandon_saved();
        self.solver.cancel();
        self.check = None;
        tracing::info!(?variant, seed, ?origin, "new game dealt");
        let recorded = self.storage.start_game(variant, seed, undo_allowed);
        let id = recorded.as_ref().ok().copied();
        let start = Start {
            id,
            elapsed: Duration::ZERO,
            undo_allowed,
        };
        self.reset_quick();
        self.board
            .begin(Session::new(variant, seed), start, self.now);
        self.enter_board();
        self.note_write(recorded.map(drop), "the new game could not be recorded");
        match origin {
            Origin::Fresh => {}
            Origin::Searched(proof) => {
                self.board.set_deal(DealCheck::Known(proof));
                if proof == Proof::Winnable {
                    self.record_proof(proof);
                }
            }
            Origin::Chosen => self.start_check(variant, seed),
        }
    }

    fn resume_game(&mut self) {
        let Some(saved) = self.saved.clone() else {
            return;
        };
        let moves = u32::try_from(saved.record.cursor).unwrap_or(u32::MAX);
        let (id, elapsed_ms) = (saved.id, saved.elapsed_ms);
        match Session::restore(saved.record) {
            Ok(session) => {
                let elapsed = Duration::from_millis(elapsed_ms);
                let undo_allowed = saved.undo_allowed;
                let start = Start {
                    id: Some(id),
                    elapsed,
                    undo_allowed,
                };
                let (variant, seed) = (session.variant(), session.seed());
                if self.board.game_id() != Some(id) {
                    self.reset_quick();
                }
                self.board.begin(session, start, self.now);
                if !undo_allowed {
                    let restarts = self.stored_restarts(id);
                    self.board.set_restarts(restarts);
                }
                self.enter_board();
                self.solver.cancel();
                self.start_check(variant, seed);
            }
            Err(error) => {
                tracing::warn!(%error, "the saved game could not be restored; removing it");
                self.discard_unusable_save(Some((id, elapsed_ms, moves)));
                self.open_modal(Modal::alert("common.error.resume"));
            }
        }
    }

    pub(super) fn apply_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.refresh_running();
        self.refresh_view();
        if let Err(error) = self.storage.save_settings(&self.settings) {
            tracing::warn!(%error, "settings could not be saved; they hold for this session");
            self.open_modal(Modal::alert("common.error.storage"));
        }
    }
}
