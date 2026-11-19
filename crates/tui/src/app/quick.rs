//! The quick choices of the running game. The settings hold the defaults of the board aids; a new
//! game copies them into `quick`, and the board toggles and the quick options dialog change only
//! that copy. The board draws with `running`, the settings with the quick choices laid over them,
//! so a default changed in Settings never reaches a game in progress. "Save as default" is the one
//! way back from the game to the settings.

use super::{App, Modal};
use crate::settings::Aids;

impl App {
    /// The aids of the running game: the defaults of the settings until a quick choice changes one.
    pub fn aids(&self) -> Aids {
        self.quick
    }

    /// A new game starts from the defaults of the settings.
    pub(super) fn reset_quick(&mut self) {
        self.set_quick(self.settings.aids());
    }

    pub(super) fn set_quick(&mut self, aids: Aids) {
        self.quick = aids;
        self.refresh_running();
    }

    pub(super) fn refresh_running(&mut self) {
        self.running = self.settings.with_aids(self.quick);
    }

    /// Copies the choices of an open quick options dialog to the game.
    pub(super) fn sync_quick(&mut self) {
        let Some(Modal::Quick(state)) = &self.modal else {
            return;
        };
        let aids = state.aids;
        if aids != self.quick {
            self.set_quick(aids);
        }
    }

    /// The quick choices become the defaults for new games and are saved with the settings.
    pub(super) fn save_quick(&mut self) {
        let settings = self.settings.with_aids(self.quick);
        self.apply_settings(settings);
        self.board.announce_defaults_saved();
    }
}
