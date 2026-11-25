//! Applying an [`Effect`]: navigation, the modal layer, starting and resuming games, settings.

use std::time::Duration;

use oxysol_core::solver::has_model;
use oxysol_core::{Session, Variant};

use super::deals::Origin;
use super::{App, Modal, Screen};
use crate::action::Action;
use crate::input::primary_key_event;
use crate::screens::Effect;
use crate::screens::board::Start;
use crate::settings::{CardSizeSetting, Settings};
use crate::solver::{DealCheck, Proof};
use crate::widgets::overlay::dock::DockState;
use crate::widgets::overlay::{HowToPlay, Rules};

impl App {
    pub(crate) fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::None => {}
            Effect::Goto(screen) => self.leave_or_ask(screen),
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
            Effect::Preview(settings) => self.preview_settings(settings),
            Effect::LeaveSettings { keep, to } => self.leave_settings(keep, to),
            Effect::Open(modal) => self.open_modal(modal),
            Effect::CardSize(size) => self.set_card_size(size),
            Effect::Quick(aids) => self.set_quick(aids),
            Effect::SaveAids => self.save_quick(),
            Effect::Board(action) => self.run_board_action(action),
            Effect::HowToPlay => self.open_how_to_play(),
            Effect::CopySeed(seed) => self.copy_seed(seed),
            Effect::CheckWinnable(targets) => self.start_stats_check(targets),
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
        if self.screen == Screen::Stats && screen != Screen::Stats {
            self.stop_stats_check();
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
        self.dock = DockState::default();
        self.board.set_covered(matches!(modal, Modal::Pause { .. }));
        self.menu.window_opened();
        self.modal = Some(modal);
        self.sync_clock();
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
        self.dock = DockState::default();
        self.board.set_covered(false);
        if self.screen == Screen::Board {
            self.board.resume_clock(self.now);
        }
    }

    fn start_game(&mut self, variant: Variant, seed: Option<u64>, undo_allowed: bool) {
        match seed {
            Some(seed) => self.begin_game(variant, seed, undo_allowed, Origin::Chosen),
            None if self.settings.winnable_only && has_model(variant) => {
                self.begin_search(variant, undo_allowed);
            }
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
            Origin::Chosen if !has_model(variant) => {}
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
                let counters = self.stored_counters(id);
                self.board.set_assist(counters.assist);
                if !undo_allowed {
                    self.board.set_restarts(counters.restarts);
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

    /// Changes the card size alone, so the quick choices of the running game are not saved with it.
    fn set_card_size(&mut self, card_size: CardSizeSetting) {
        let settings = Settings {
            card_size,
            ..self.settings.clone()
        };
        self.apply_settings(settings);
    }

    pub(super) fn apply_settings(&mut self, settings: Settings) {
        self.show_settings(settings);
        self.persist_settings();
    }

    /// Makes `settings` the ones in use: the console, the theme and the running view follow. Nothing
    /// is saved.
    pub(super) fn show_settings(&mut self, mut settings: Settings) {
        let full_screen_changed = self.settings.full_screen != settings.full_screen;
        let window_size_changed = self.settings.window_size != settings.window_size;
        let console_font_changed = self.settings.console_font != settings.console_font
            || self.settings.console_face != settings.console_face
            || self.settings.console_font_size != settings.console_font_size;
        let priority_changed = self.settings.process_priority != settings.process_priority;
        let check = self.needs_graphics_check(&settings);
        if check {
            settings.glyph_mode = self.settings.glyph_mode;
        }
        let was_full_screen = self.full_screen.active;
        self.settings = settings;
        if full_screen_changed {
            self.full_screen_setting_changed();
        }
        let left_full_screen = was_full_screen && !self.full_screen.active;
        if window_size_changed || left_full_screen {
            self.sync_window_size();
        }
        if console_font_changed {
            self.console_font_setting_changed();
        }
        if priority_changed {
            self.priority_setting_changed();
        }
        if check {
            self.open_graphics_check();
        }
        self.refresh_running();
        self.refresh_view();
    }

    /// Saves the settings through storage. The first failure since the last successful write is
    /// logged and shown as an alert; later failures stay silent until a write succeeds again.
    pub(super) fn persist_settings(&mut self) {
        let saved = match &self.interface_baseline {
            Some(baseline) => self.settings.with_interface_of(baseline),
            None => self.settings.clone(),
        };
        let Err(error) = self.storage.save_settings(&saved) else {
            self.settings_write_failed = false;
            return;
        };
        if self.settings_write_failed {
            return;
        }
        self.settings_write_failed = true;
        tracing::warn!(%error, "settings could not be saved; they hold for this session");
        self.open_modal(Modal::alert("common.error.storage"));
    }
}
