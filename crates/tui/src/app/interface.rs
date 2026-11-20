//! The preview of the Interface section of the settings (P3-29): a change shows at once but is
//! saved only when the player keeps it on leaving the settings screen. `interface_baseline` holds
//! the settings as they were saved before the first preview; while it is set, storage receives the
//! baseline's interface fields, so a closed program never keeps a preview.

use super::{App, Modal, Screen};
use crate::screens::Effect;
use crate::settings::Settings;

const KEEP_PROMPT: &str = "settings.confirm.keep_interface";

impl App {
    /// Shows `settings` without saving them. The first preview remembers what to go back to; a
    /// preview that ends where it started is no preview.
    pub(super) fn preview_settings(&mut self, settings: Settings) {
        let baseline = self
            .interface_baseline
            .get_or_insert_with(|| self.settings.clone())
            .clone();
        self.show_settings(settings);
        if self.settings.with_interface_of(&baseline) == self.settings {
            self.interface_baseline = None;
        }
        self.sync_preview_note();
    }

    /// Leaves the settings screen for `to`. With a preview open the player is asked first.
    pub(super) fn leave_or_ask(&mut self, to: Screen) {
        if self.screen == Screen::Settings && to != Screen::Settings && self.previewing() {
            let keep = Effect::LeaveSettings { keep: true, to };
            let undo = Effect::LeaveSettings { keep: false, to };
            self.open_modal(Modal::confirm_or(KEEP_PROMPT, keep, undo));
            return;
        }
        self.goto(to);
    }

    /// Ends the preview: the interface changes are saved (`keep`) or undone, then `to` opens.
    pub(super) fn leave_settings(&mut self, keep: bool, to: Screen) {
        let baseline = self.interface_baseline.take();
        if let Some(baseline) = baseline.filter(|_| !keep) {
            let restored = self.settings.with_interface_of(&baseline);
            self.show_settings(restored);
        }
        self.persist_settings();
        self.sync_preview_note();
        self.goto(to);
    }

    pub(super) fn previewing(&self) -> bool {
        self.interface_baseline.is_some()
    }

    pub(super) fn sync_preview_note(&mut self) {
        let previewing = self.previewing();
        self.settings_screen.set_previewing(previewing);
    }
}
