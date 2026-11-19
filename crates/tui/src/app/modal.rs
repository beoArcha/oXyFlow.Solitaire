//! The modal layer's state: which dialog is open and what confirming it does.

use super::search::SearchProgress;
use crate::screens::Effect;
use crate::widgets::overlay::{CommandLine, HowToPlay};

/// A dialog over the current screen. While one is open it takes all input and the clock stops.
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    Help,
    /// The rules of one game, with tabs for the others.
    HowToPlay(HowToPlay),
    /// The options of `oxysol --help`, the third tab of the help window.
    CommandLine(CommandLine),
    Pause,
    /// A question; `on_yes` is applied only when the player confirms. Focus starts on "No".
    Confirm {
        prompt: &'static str,
        on_yes: Box<Effect>,
        yes_focused: bool,
    },
    /// A notice with one button; `message` is a localization key.
    Alert {
        message: &'static str,
    },
    /// The search for a winnable deal: progress, then the choices once the cap is reached.
    Search(SearchProgress),
    /// The win screen; `menu_focused` picks which of its two buttons has the focus.
    Win {
        menu_focused: bool,
    },
}

impl Modal {
    /// A notice showing the text of the localization key `message`.
    pub fn alert(message: &'static str) -> Self {
        Modal::Alert { message }
    }

    /// The win screen, with focus on "New game".
    pub fn win() -> Self {
        Modal::Win {
            menu_focused: false,
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
