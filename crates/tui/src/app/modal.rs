//! The modal layer's state: which dialog is open and what confirming it does.

use super::search::SearchProgress;
use crate::screens::Effect;
use crate::widgets::overlay::{
    CommandLine, GuideWindow, HelpKeys, HowToPlay, MovesWindow, PersonalBest, QuickOptions,
};

/// A dialog over the current screen. While one is open it takes all input and the clock stops.
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    /// The keys of the help window, with the scroll of its Keys tab.
    Help(HelpKeys),
    /// The rules of one game, with tabs for the others.
    HowToPlay(HowToPlay),
    /// The options of `oxysol --help`, the third tab of the help window.
    CommandLine(CommandLine),
    /// The content of the guide panel as a window, on a screen too narrow for the panel.
    Guide(GuideWindow),
    /// The entries of the possible moves panel as a window, on a screen too narrow for the panel.
    Moves(MovesWindow),
    /// The aids of the running game, switched at once; "Save as default" keeps them for new games.
    Quick(QuickOptions),
    /// The pause dialog; `restart_focused` is true when the Restart button has focus.
    Pause {
        undo_allowed: bool,
        restart_focused: bool,
    },
    /// A question; `on_yes` is applied only when the player confirms. Focus starts on "No".
    Confirm {
        prompt: &'static str,
        on_yes: Box<Effect>,
        yes_focused: bool,
    },
    /// A notice with one button; `message` is a localization key.
    Alert { message: &'static str },
    /// The search for a winnable deal: progress, then the choices once the cap is reached.
    Search(SearchProgress),
    /// The win screen; `focus` is the index of the focused button and `best` the personal bests
    /// the game set (filled in when the dialog opens).
    Win { focus: usize, best: PersonalBest },
}

impl Modal {
    /// The help window on its Keys tab, at the top.
    pub fn help() -> Self {
        Modal::Help(HelpKeys::default())
    }

    /// A notice showing the text of the localization key `message`.
    pub fn alert(message: &'static str) -> Self {
        Modal::Alert { message }
    }

    /// The pause dialog, with focus on "Continue".
    pub fn pause(undo_allowed: bool) -> Self {
        Modal::Pause {
            undo_allowed,
            restart_focused: false,
        }
    }

    /// The win screen, with focus on "New game" and no personal best yet.
    pub fn win() -> Self {
        Modal::Win {
            focus: 0,
            best: PersonalBest::default(),
        }
    }

    /// A question showing the text of the localization key `prompt`.
    pub fn confirm(prompt: &'static str, on_yes: Effect) -> Self {
        Modal::Confirm {
            prompt,
            on_yes: Box::new(on_yes),
            yes_focused: false,
        }
    }
}
