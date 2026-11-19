//! The pause dialog. It covers the whole board, so no card is visible while the clock is stopped.

use super::dialog::{Button, Text};
use crate::i18n::t;

pub const RESTART: usize = 1;

/// The pause dialog: Continue and Restart buttons.
pub fn text(restart_focused: bool) -> Text {
    Text {
        title: t("common.dialog.pause_title").into_owned(),
        lines: vec![t("common.dialog.pause_body").into_owned()],
        buttons: vec![
            Button {
                label: t("help.overlay.continue").into_owned(),
                focused: !restart_focused,
            },
            Button {
                label: t("help.overlay.restart").into_owned(),
                focused: restart_focused,
            },
        ],
    }
}
