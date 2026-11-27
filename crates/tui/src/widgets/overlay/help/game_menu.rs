//! The Game menu topic of the Keys tab: the keys that only the game list reads itself. They are
//! not in the key table, because they have no action, so the rows are listed here by hand.

use super::{Row, Section};
use crate::i18n::t;

const GAME_MENU: [(&str, &str); 6] = [
    ("help.game_menu_keys.page", "help.game_menu.page"),
    ("help.game_menu_keys.home_end", "help.game_menu.home_end"),
    ("help.game_menu_keys.search", "help.game_menu.search"),
    ("help.game_menu_keys.winnable", "help.game_menu.winnable"),
    ("help.game_menu_keys.fold", "help.game_menu.fold"),
    (
        "help.game_menu_keys.fold_heading",
        "help.game_menu.fold_heading",
    ),
];

/// The Game menu topic, placed before the Mouse topic.
pub(super) fn section() -> Section {
    Section {
        title: t("help.group.game_menu").into_owned(),
        rows: GAME_MENU
            .iter()
            .map(|(keys, text)| Row {
                keys: t(keys).into_owned(),
                text: t(text).into_owned(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests;
