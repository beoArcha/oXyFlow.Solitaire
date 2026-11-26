//! The no-moves-left dialog: shown once when a move leaves the game not won and with no legal
//! moves. Offers Undo (when undo is allowed and possible), Restart, New game and Menu.

use super::dialog::{Button, Text};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::t;
use crate::screens::Effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Undo,
    Restart,
    NewGame,
    Menu,
}

const WITH_UNDO: [Choice; 4] = [Choice::Undo, Choice::Restart, Choice::NewGame, Choice::Menu];
const WITHOUT_UNDO: [Choice; 3] = [Choice::Restart, Choice::NewGame, Choice::Menu];

fn choices(can_undo: bool) -> &'static [Choice] {
    if can_undo { &WITH_UNDO } else { &WITHOUT_UNDO }
}

const fn label_key(choice: Choice) -> &'static str {
    match choice {
        Choice::Undo => "board.moves.label.undo",
        Choice::Restart => "help.overlay.restart",
        Choice::NewGame => "help.overlay.new_game",
        Choice::Menu => "help.overlay.menu",
    }
}

const fn effect(choice: Choice) -> Effect {
    match choice {
        Choice::Undo => Effect::Board(Action::Undo),
        Choice::Restart => Effect::Restart,
        Choice::NewGame => Effect::Goto(Screen::GameSelect),
        Choice::Menu => Effect::Goto(Screen::MainMenu),
    }
}

/// Number of buttons in the dialog for `can_undo`.
pub fn button_count(can_undo: bool) -> usize {
    choices(can_undo).len()
}

/// The effect of pressing button `index` in the no-moves-left dialog.
pub fn effect_at(can_undo: bool, index: usize) -> Effect {
    choices(can_undo)
        .get(index)
        .copied()
        .map_or(Effect::None, effect)
}

/// The no-moves-left dialog with `focus` as the index of the focused button.
pub fn text(can_undo: bool, focus: usize) -> Text {
    Text {
        title: t("board.no_moves.title").into_owned(),
        lines: vec![t("board.no_moves.body").into_owned()],
        buttons: choices(can_undo)
            .iter()
            .enumerate()
            .map(|(index, choice)| Button {
                label: t(label_key(*choice)).into_owned(),
                focused: index == focus,
            })
            .collect(),
    }
}
