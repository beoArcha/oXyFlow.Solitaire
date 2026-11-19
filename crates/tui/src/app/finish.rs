//! The auto-finish clock of the board: runs once per event, while the board is shown with no modal.

use super::{App, Screen};
use crate::screens::Effect;

impl App {
    pub(super) fn advance_finish(&mut self) -> Effect {
        if self.screen != Screen::Board || self.modal.is_some() {
            return Effect::None;
        }
        self.board.advance(self.now, self.settings.auto_finish)
    }
}
