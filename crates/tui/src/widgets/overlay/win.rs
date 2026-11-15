//! The win screen: the solved game, its elapsed time and the choice of "New game" or "Menu".

use std::time::Duration;

use super::dialog::{Button, Text};
use crate::clock::format_elapsed;
use crate::i18n::{t, tf};

/// The win screen. `menu_focused` moves the focus from "New game" to "Menu".
pub fn text(elapsed: Duration, menu_focused: bool) -> Text {
    Text {
        title: t("common.dialog.win_title").into_owned(),
        lines: vec![
            t("common.dialog.win_body").into_owned(),
            tf(
                "help.overlay.win_time",
                &[("time", &format_elapsed(elapsed))],
            ),
        ],
        buttons: vec![
            Button {
                label: t("help.overlay.new_game").into_owned(),
                focused: !menu_focused,
            },
            Button {
                label: t("help.overlay.menu").into_owned(),
                focused: menu_focused,
            },
        ],
    }
}
