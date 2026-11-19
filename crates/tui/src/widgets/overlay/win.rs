//! The win screen: the solved game, its elapsed time, its move count, a line for each personal best
//! and the choice of "New game", "Same seed" or "Menu".

use std::time::Duration;

use oxysol_core::Variant;

use super::dialog::{Button, Text};
use crate::clock::format_elapsed;
use crate::i18n::{t, tf};
use crate::screens::variant_label_key;

/// The buttons of the win screen, left to right.
pub const BUTTONS: usize = 3;
/// The index of "New game".
pub const NEW_GAME: usize = 0;
/// The index of "Same seed".
pub const SAME_SEED: usize = 1;
/// The index of "Menu".
pub const MENU: usize = 2;

/// Whether the time and the move count of the game beat its variant's earlier wins.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PersonalBest {
    pub time: bool,
    pub moves: bool,
}

/// The win screen of the game `variant` (`None` without a game, which has no personal best);
/// `focus` is the index of the focused button.
pub fn text(
    elapsed: Duration,
    moves: usize,
    variant: Option<Variant>,
    focus: usize,
    best: PersonalBest,
) -> Text {
    let mut lines = vec![
        t("common.dialog.win_body").into_owned(),
        tf(
            "help.overlay.win_time",
            &[("time", &format_elapsed(elapsed))],
        ),
        tf("common.dialog.win_moves", &[("moves", &moves.to_string())]),
    ];
    if let Some(variant) = variant {
        let game = t(variant_label_key(variant)).into_owned();
        if best.time {
            lines.push(tf("help.overlay.best_time", &[("game", &game)]));
        }
        if best.moves {
            lines.push(tf("help.overlay.best_moves", &[("game", &game)]));
        }
    }
    Text {
        title: t("common.dialog.win_title").into_owned(),
        lines,
        buttons: [
            ("help.overlay.new_game", NEW_GAME),
            ("help.overlay.same_seed", SAME_SEED),
            ("help.overlay.menu", MENU),
        ]
        .into_iter()
        .map(|(key, index)| Button {
            label: t(key).into_owned(),
            focused: index == focus,
        })
        .collect(),
    }
}
