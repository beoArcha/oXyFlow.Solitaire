//! The font face and the font size the player chooses for the classic Windows console (P3-29). Only
//! that console lets the game change them; every other terminal keeps its own font and its own
//! zoom shortcut.

use serde::{Deserialize, Serialize};

/// The font face the game asks the classic console for. Each one covers the WGL4 symbols the
/// Unicode glyph set uses. A face the machine does not have is reported as not installed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsoleFace {
    #[default]
    Default,
    Consolas,
    LucidaConsole,
    CascadiaMono,
}

impl ConsoleFace {
    /// Every choice, in the order of the settings screen.
    pub const ALL: [ConsoleFace; 4] = [
        ConsoleFace::Default,
        ConsoleFace::Consolas,
        ConsoleFace::LucidaConsole,
        ConsoleFace::CascadiaMono,
    ];

    /// The family name to ask the console for; `None` leaves the face the console had.
    pub const fn name(self) -> Option<&'static str> {
        match self {
            ConsoleFace::Default => None,
            ConsoleFace::Consolas => Some("Consolas"),
            ConsoleFace::LucidaConsole => Some("Lucida Console"),
            ConsoleFace::CascadiaMono => Some("Cascadia Mono"),
        }
    }
}

/// The font heights in pixels the settings offer; `0` leaves the size the console had.
pub const CONSOLE_FONT_SIZES: [u8; 9] = [0, 12, 14, 16, 18, 20, 24, 28, 32];
