//! The card back options: Theme, Monokai, Dracula.

use serde::{Deserialize, Serialize};

/// Which card back pattern and palette to use for face-down cards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardBack {
    #[default]
    Theme,
    Monokai,
    Dracula,
}

impl CardBack {
    /// Every card back style, in the order of the settings screen.
    pub const ALL: [CardBack; 3] = [CardBack::Theme, CardBack::Monokai, CardBack::Dracula];

    /// Localization key of the card back name.
    pub const fn label_key(self) -> &'static str {
        match self {
            CardBack::Theme => "settings.card_back.theme",
            CardBack::Monokai => "settings.card_back.monokai",
            CardBack::Dracula => "settings.card_back.dracula",
        }
    }
}
