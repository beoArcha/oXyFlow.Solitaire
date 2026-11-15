//! The pause dialog. It covers the whole board, so no card is visible while the clock is stopped.

use super::dialog::{Button, Text};
use crate::i18n::t;

/// The pause dialog: a notice with one button that resumes the game.
pub fn text() -> Text {
    Text {
        title: t("common.dialog.pause_title").into_owned(),
        lines: vec![t("common.dialog.pause_body").into_owned()],
        buttons: vec![Button {
            label: t("help.overlay.continue").into_owned(),
            focused: true,
        }],
    }
}
